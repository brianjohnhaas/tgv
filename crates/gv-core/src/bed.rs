use crate::{
    contig_header::ContigHeader,
    error::TGVError,
    intervals::{GenomeInterval, SortedIntervalCollection},
};
use noodles::bed::{self};
use noodles::bgzf;
use std::{fs::File, path::Path};

#[derive(Debug, Clone)]
pub struct BEDInterval {
    contig_index: usize,

    pub index: usize,

    start: u64,
    end: u64,

    record: bed::Record<3>,
}

impl BEDInterval {
    pub fn new(
        record: bed::Record<3>,
        index: usize,
        contig_header: &ContigHeader,
    ) -> Result<Self, TGVError> {
        let start = record.feature_start()?.get() as u64; // Noodles already converted to 1-based, inclusive
        Ok(Self {
            contig_index: contig_header
                .try_get_index_by_str(&record.reference_sequence_name().to_string())?,
            index,
            start, // BED start is 0-based, inclusive
            end: match record.feature_end() {
                Some(end) => end?.get() as u64,
                None => start, // BED end is 0-based, exclusive
            },
            record,
        })
    }

    pub fn describe(&self) -> String {
        format!(
            "BED interval: {}:{}-{}",
            self.record.reference_sequence_name(),
            self.start,
            self.end
        )
    }

    pub fn exon_segments(&self) -> Result<Option<Vec<(u64, u64)>>, TGVError> {
        let other_fields = self.record.other_fields();
        let parse_u64_field = |index: usize| -> Result<u64, TGVError> {
            other_fields
                .get(index)
                .ok_or_else(|| TGVError::ParsingError(format!("Missing BED field at index {}", index)))
                .and_then(|field| {
                    std::str::from_utf8(field.as_ref())
                        .map_err(|e| {
                            TGVError::ParsingError(format!("Invalid UTF-8 in BED field: {}", e))
                        })?
                        .parse::<u64>()
                        .map_err(Into::into)
                })
        };
        let parse_u64_list_field = |index: usize| -> Result<Vec<u64>, TGVError> {
            let field = other_fields
                .get(index)
                .ok_or_else(|| TGVError::ParsingError(format!("Missing BED field at index {}", index)))?;
            let field = std::str::from_utf8(field.as_ref())
                .map_err(|e| TGVError::ParsingError(format!("Invalid UTF-8 in BED field: {}", e)))?;

            field
                .trim_end_matches(',')
                .split(',')
                .filter(|value| !value.is_empty())
                .map(|value| value.parse::<u64>().map_err(Into::into))
                .collect()
        };

        if other_fields.len() < 9 {
            return Ok(None);
        }

        let block_count = parse_u64_field(6)?;
        let block_sizes = parse_u64_list_field(7)?;
        let block_starts = parse_u64_list_field(8)?;

        if block_count == 0 {
            return Ok(None);
        }

        if block_sizes.len() != block_count as usize || block_starts.len() != block_count as usize {
            return Err(TGVError::ParsingError(format!(
                "Invalid BED12 block structure for interval {}:{}-{}",
                self.record.reference_sequence_name(),
                self.start,
                self.end
            )));
        }

        let segments = block_sizes
            .into_iter()
            .zip(block_starts)
            .map(|(block_size, block_start)| {
                let start = self.start + block_start;
                let end = start + block_size.saturating_sub(1);
                (start, end)
            })
            .collect();

        Ok(Some(segments))
    }
}

impl GenomeInterval for BEDInterval {
    fn contig_index(&self) -> usize {
        self.contig_index
    }

    fn start(&self) -> u64 {
        self.start
    }

    fn end(&self) -> u64 {
        self.end
    }
}
#[derive(Debug, Clone)]
pub struct BEDRepository {
    pub bed_path: String,
}

impl BEDRepository {
    pub fn read_bed(
        &self,
        contig_header: &ContigHeader,
    ) -> Result<SortedIntervalCollection<BEDInterval>, TGVError> {
        let path = Path::new(&self.bed_path);
        let mut record = bed::Record::default();
        let mut records = Vec::new();
        let mut index = 0;

        if is_bgzf_bed_path(path) {
            let file = File::open(path)?;
            let bgzf_reader = bgzf::io::Reader::new(file);
            let mut reader = bed::io::reader::Builder::<3>::default().build_from_reader(bgzf_reader);

            while reader.read_record(&mut record)? != 0 {
                records.push(BEDInterval::new(record.clone(), index, contig_header)?);
                index += 1;
            }
        } else {
            let mut reader = bed::io::reader::Builder::<3>::default().build_from_path(path)?;

            while reader.read_record(&mut record)? != 0 {
                records.push(BEDInterval::new(record.clone(), index, contig_header)?);
                index += 1;
            }
        }

        SortedIntervalCollection::new(records)
    }
}

fn is_bgzf_bed_path(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| matches!(ext, "gz" | "bgz"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        contig_header::{ContigHeader, ContigSource},
        reference::Reference,
    };
    use std::io::Write;

    fn contig_header_with_test_contig() -> ContigHeader {
        let mut contig_header = ContigHeader::new(Reference::NoReference);
        contig_header.update_or_add_contig(
            "CHD2--LINC01578".to_string(),
            Some(68_151),
            Vec::new(),
            ContigSource::Sequence,
        );
        contig_header
    }

    #[test]
    fn reads_bgzf_bed_input() -> Result<(), TGVError> {
        let temp_dir = tempfile::tempdir()?;
        let bed_path = temp_dir.path().join("finspector.bed.gz");

        {
            let file = File::create(&bed_path)?;
            let mut writer = bgzf::io::Writer::new(file);
            writer.write_all(
                b"CHD2--LINC01578\t57496\t67151\tname\t0\t+\t57496\t67151\t0\t5\t480,70,86,69,1978\t0,2808,3462,6608,7677\n",
            )?;
            writer.finish()?;
        }

        let repository = BEDRepository {
            bed_path: bed_path.to_string_lossy().into_owned(),
        };
        let contig_header = contig_header_with_test_contig();

        let intervals = repository.read_bed(&contig_header)?;
        assert_eq!(intervals.overlapping(0, 57_497, 67_151)?.len(), 1);

        Ok(())
    }

    #[test]
    fn detects_bgzf_extensions() {
        assert!(is_bgzf_bed_path(Path::new("input.bed.gz")));
        assert!(is_bgzf_bed_path(Path::new("input.bed.bgz")));
        assert!(!is_bgzf_bed_path(Path::new("input.bed")));
    }
}
