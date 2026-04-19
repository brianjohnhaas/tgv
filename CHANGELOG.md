# Changelog

## unreleased

- Improve BED track loading and rendering for local custom references.
- Support compressed `.bed.gz` and `.bed.bgz` BED inputs.
- Render BED intervals with visible glyphs instead of subtle background-only shading.
- Render BED12 exon blocks with `█` and intronic spans with `-`.
- Show BED features on multiple rows instead of collapsing them into a single line.
- Size the BED track dynamically based on the visible region while preserving alignment space.
- Add a `BED` label and a `+N` overflow indicator to the BED track.
- Add `CODEX.md` with a project overview, build and run instructions, and onboarding notes.

## 0.0.6

- Cache support

## 0.0.5

- Supports UCSC accession IDs
- Contig switching `{` / `}`
- List contigs `:ls`
- UCSC EU server `tgv --host eu`

## 0.0.4

- More reference genomes
- Fix cigar display bug
- Architecture improvement

## 0.0.3

Stability improvements.

## 0.0.2

Major update on UI, testing, CI, state handling.

- New UI elements: half-character base render, cytobands, high-level zoom, non-cds exons, genome lenght upper bound
- Stability improvements
- Integration test!
- Releases with Github Action

## 0.0.1

A minimum app runs. Has bugs. Miss key features. Miss tests.
