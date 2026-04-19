use crate::{
    layout::{AlignmentView, OnScreenCoordinate},
    rendering::colors::Palette,
};
use gv_core::{bed::BEDInterval, error::TGVError, intervals::GenomeInterval, state::State};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
};

pub fn render_bed(
    area: &Rect,
    buf: &mut Buffer,
    state: &State,
    alignment_view: &AlignmentView,
    pallete: &Palette,
) -> Result<(), TGVError> {
    render_bed_label(area, buf, pallete);

    let region = alignment_view.region(area);
    let intervals =
        state
            .bed_intervals
            .overlapping(region.contig_index(), region.start(), region.end())?; // FIXME: wasteful calculation here

    if !intervals.is_empty() {
        render_bed_intervals(area, buf, intervals, alignment_view, pallete)?;
    }

    Ok(())
}

fn render_bed_intervals(
    area: &Rect,
    buf: &mut Buffer,
    intervals: Vec<&BEDInterval>,
    alignment_view: &AlignmentView,
    palette: &Palette,
) -> Result<(), TGVError> {
    let mut row_right_edges: Vec<Option<u64>> = vec![None; area.height as usize];
    let mut hidden_count = 0usize;

    for interval in intervals {
        let Some(row) = first_available_row(interval, &mut row_right_edges) else {
            hidden_count += 1;
            continue;
        };

        let color = if interval.start() % 2 == 0 {
            palette.BED1
        } else {
            palette.BED2
        };

        render_interval_structure(area, buf, interval, row as u16, alignment_view, color)?;
    }

    if hidden_count > 0 {
        render_overflow_indicator(area, buf, hidden_count, palette);
    }

    Ok(())
}

fn render_interval_structure(
    area: &Rect,
    buf: &mut Buffer,
    interval: &BEDInterval,
    row: u16,
    alignment_view: &AlignmentView,
    color: ratatui::style::Color,
) -> Result<(), TGVError> {
    let style = Style::default().fg(color).add_modifier(Modifier::BOLD);

    match interval.exon_segments()? {
        Some(exons) if !exons.is_empty() => {
            for window in exons.windows(2) {
                let intron_start = window[0].1.saturating_add(1);
                let intron_end = window[1].0.saturating_sub(1);
                if intron_start <= intron_end {
                    render_genomic_span(area, buf, intron_start, intron_end, row, alignment_view, "-", style);
                }
            }

            for (start, end) in exons {
                render_genomic_span(area, buf, start, end, row, alignment_view, "█", style);
            }
        }
        _ => {
            render_genomic_span(
                area,
                buf,
                interval.start(),
                interval.end(),
                row,
                alignment_view,
                "█",
                style,
            );
        }
    }

    Ok(())
}

fn render_genomic_span(
    area: &Rect,
    buf: &mut Buffer,
    start: u64,
    end: u64,
    row: u16,
    alignment_view: &AlignmentView,
    glyph: &str,
    style: Style,
) {
    let start = alignment_view.onscreen_x_coordinate(start, area);
    let end = alignment_view.onscreen_x_coordinate(end, area);

    if let Some((x, length)) = OnScreenCoordinate::onscreen_start_and_length(&start, &end, area) {
        buf.set_string(
            area.x + x,
            area.y + row,
            glyph.repeat(length as usize),
            style,
        );
    }
}

fn first_available_row<T: GenomeInterval>(
    interval: &T,
    row_right_edges: &mut [Option<u64>],
) -> Option<usize> {
    for (row, right_edge) in row_right_edges.iter_mut().enumerate() {
        match right_edge {
            Some(last_end) if interval.start() <= *last_end => continue,
            _ => {
                *right_edge = Some(interval.end());
                return Some(row);
            }
        }
    }

    None
}

fn render_bed_label(area: &Rect, buf: &mut Buffer, palette: &Palette) {
    if area.width < 3 || area.height == 0 {
        return;
    }

    buf.set_string(
        area.x,
        area.y,
        "BED",
        Style::default().fg(palette.BED2).add_modifier(Modifier::BOLD),
    );
}

fn render_overflow_indicator(area: &Rect, buf: &mut Buffer, hidden_count: usize, palette: &Palette) {
    let indicator = format!("+{}", hidden_count);
    let width = indicator.len() as u16;

    if area.width < width {
        return;
    }

    let x = area.right().saturating_sub(width);
    let y = area.bottom().saturating_sub(1);

    buf.set_string(
        x,
        y,
        indicator,
        Style::default().fg(palette.BED2).add_modifier(Modifier::BOLD),
    );
}
