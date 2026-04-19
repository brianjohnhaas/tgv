# tgv codex notes

## project overview

This repository is a small Rust workspace for a terminal genome viewer.

At a high level:

- `crates/gv-core` contains the core domain logic, data access, interval math, reference handling, and UCSC/cache integration.
- `crates/tgv` contains the terminal application, including the CLI, app loop, input handling, layout, and rendering.

The main runtime flow is:

1. Parse CLI arguments in `crates/tgv/src/main.rs`.
2. Convert the CLI into app settings in `crates/tgv/src/settings.rs`.
3. Build repositories and the contig header in `crates/gv-core/src/repository.rs`.
4. Initialize the shared application state in `crates/gv-core/src/state.rs`.
5. Start the TUI event loop in `crates/tgv/src/app.rs`.

The `Repository` type decides which optional services are active:

- alignment repository for BAM input.
- sequence service for UCSC, indexed FASTA, or 2bit reference access.
- track service for gene and cytoband data.
- optional VCF and BED readers.

The `State` type is the main in-memory model for the current viewport. It owns the loaded sequence, alignments, track data, variants, BED intervals, contig metadata, and user messages. The `App` coordinates rendering and lazy data loading based on the current focus and zoom level.

## crate boundaries

### `gv-core`

This crate is the backend and domain layer.

Important areas:

- `alignment/`: BAM and CRAM repository code, alignment parsing, read display logic, and coverage.
- `sequence/`: indexed FASTA, 2bit, and UCSC API sequence access.
- `tracks/`: UCSC database/API/local-cache track access, plus the downloader that materializes local cache data.
- `reference.rs`: reference parsing and resolution.
- `contig_header.rs`: contig identity and alias mapping across multiple sources.
- `state.rs`: navigation, cache loading, and state transitions.

### `tgv`

This crate is the terminal frontend.

Important areas:

- `main.rs`: entry point and top-level command handling.
- `settings.rs`: CLI parsing and validation.
- `app.rs`: app lifecycle, event loop, message handling, and lazy loading.
- `layout.rs`: viewport math and screen area management.
- `register.rs` and `mouse.rs`: keyboard and mouse input handling.
- `rendering/`: drawing for alignments, tracks, sequence, status bar, help, and related views.

## current maturity notes

The codebase is coherent and buildable, but it is still evolving.

Notable characteristics:

- There are many `TODO` and `FIXME` markers across core paths.
- There are many `unwrap()` calls in correctness-sensitive code, especially in alignment parsing and downloader code.
- CI currently uses `cargo test`, but the repo instructions prefer `cargo nextest run`.
- The workspace builds successfully with `cargo check`, but it currently emits many warnings.

Two workspace-level warnings surfaced immediately during `cargo check`:

- the `release` profile is defined in `crates/tgv/Cargo.toml`, but workspace builds only honor profiles in the root manifest.
- the workspace root does not specify an explicit Cargo resolver even though the member crates use edition 2024.

## build and run

Run commands from the repository root:

```bash
cd /home/unix/bhaas/projects/GITHUB/tgv
```

### fast compile check

Use this while making changes:

```bash
cargo check
```

This is usually the fastest way to verify that the workspace still compiles.

### debug build

Build the workspace in debug mode:

```bash
cargo build
```

### release build

Build an optimized binary:

```bash
cargo build --release
```

If the release profile is moved to the workspace root later, that build will fully respect the intended release settings.

### run the app

Run the `tgv` binary through Cargo:

```bash
cargo run -p tgv -- --help
```

The `--` separates Cargo arguments from program arguments.

Examples:

```bash
cargo run -p tgv -- tests/data/covid.sorted.bam --no-reference -r MN908947.3:100 --offline
cargo run -p tgv -- -g wuhCor1 --offline --cache-dir tests/data/cache
```

After a release build, you can run the binary directly:

```bash
./target/release/tgv --help
```

## tests

The repo instructions prefer `nextest` for unit and integration tests:

```bash
cargo nextest run
```

Other useful commands:

```bash
cargo nextest run --all-features
cargo nextest run --profile ci
cargo test --doc
```

## recommended working loop

For day-to-day development, this is a reasonable sequence:

```bash
cargo check
cargo nextest run
cargo run -p tgv -- --help
```

If you are changing CLI parsing, repository setup, layout math, or rendering behavior, it is also worth running one or two representative app commands against the fixtures in `crates/tgv/tests/data`.

## first 30 minutes in this repo

If you are new to Rust, the main goal at first is not to understand every type or lifetime. The goal is to learn how this repo is laid out and how changes move through it.

### 1. make sure the repo builds

Start with:

```bash
cargo check
```

That confirms the workspace is set up correctly and gives you fast feedback while you are learning the code.

### 2. run the program once

Use a fixture so you do not need to guess about input files:

```bash
cargo run -p tgv -- crates/tgv/tests/data/covid.sorted.bam --no-reference -r MN908947.3:100 --offline
```

This gives you a concrete mental model of what the application actually does.

### 3. read the entry point

Open `crates/tgv/src/main.rs` first. That file shows:

- how the CLI is parsed.
- how commands like `download` and `list` are handled.
- where the `App` is constructed.
- where the TUI starts running.

If you are only going to read one file first, read that one.

### 4. follow settings into runtime state

Then read these files in order:

1. `crates/tgv/src/settings.rs`
2. `crates/gv-core/src/repository.rs`
3. `crates/gv-core/src/state.rs`
4. `crates/tgv/src/app.rs`

That sequence shows how command-line input turns into repository services, then into state, then into the running app.

### 5. choose the right crate for the problem

When you want to change behavior, ask which side of the boundary the problem is on:

- If the issue is file parsing, coordinates, reference lookup, tracks, or data loading, start in `gv-core`.
- If the issue is keybindings, mouse behavior, layout, drawing, or terminal behavior, start in `tgv`.

This split is one of the most important things to understand in the repo.

### 6. use `cargo check` aggressively

In Rust, the compiler is part of the development loop. It is normal to make a small change, run `cargo check`, fix the reported type errors, and repeat.

That is not a sign that something is wrong. That is the normal workflow.

### 7. read compiler errors from the top

When Rust reports many errors, the first one is often the real cause and the rest are downstream noise. Fix the first meaningful error, then run `cargo check` again.

### 8. run tests before and after changes

This repo prefers:

```bash
cargo nextest run
```

If you are changing something narrow, it is still useful to run the full test suite at the start so you know whether failures are new or pre-existing.

### 9. do not panic about unfamiliar syntax

You will see patterns like:

- `Result<T, E>` for fallible operations.
- `Option<T>` for values that may be absent.
- `match` for branching on enums.
- `impl` blocks for methods on types.
- `?` to propagate errors.

Those are core Rust idioms. You do not need to master all of them before making small targeted changes in this repo.

### 10. good first places to make small changes

For low-risk exploration, look at:

- CLI validation and messages in `crates/tgv/src/settings.rs`.
- rendering details in `crates/tgv/src/rendering/`.
- warning cleanup for clearly unused imports or variables.
- targeted TODO or FIXME markers that are local and isolated.

Avoid starting with the alignment parsing code unless you specifically want a deeper and more correctness-sensitive part of the codebase.

## useful sample invocations

Show help:

```bash
cargo run -p tgv -- --help
```

Run against the included COVID BAM fixture without a reference:

```bash
cargo run -p tgv -- crates/tgv/tests/data/covid.sorted.bam --no-reference -r MN908947.3:100 --offline
```

Run with the cached `wuhCor1` reference fixture:

```bash
cargo run -p tgv -- -g wuhCor1 --offline --cache-dir crates/tgv/tests/data/cache
```

## practical notes for future work

- Prefer starting from `gv-core` when debugging data correctness issues.
- Prefer starting from `tgv/src/app.rs` and `tgv/src/rendering/` when debugging user-visible behavior.
- If a change touches serialization, input formats, or cache layout, trace backward and forward compatibility explicitly.
- If a change touches alignment parsing or coordinate math, expect off-by-one and reference/contig alias edge cases.

## small Rust glossary

This is a short, practical glossary for the Rust concepts you will see most often in this repo.

### `Result<T, E>`

This means an operation can either succeed or fail.

- `Ok(value)` means success.
- `Err(error)` means failure.

Example idea:

```rust
fn parse_reference(s: &str) -> Result<Reference, TGVError>
```

That means the function returns either a `Reference` or a `TGVError`.

### `Option<T>`

This means a value may or may not be present.

- `Some(value)` means the value exists.
- `None` means it does not.

In this repo, optional repositories and services often use `Option<T>`.

### `match`

This is Rust's main pattern-matching construct. It is similar to a `switch`, but much more expressive.

Example shape:

```rust
match self.reference {
    Reference::Hg19 => { ... }
    Reference::Hg38 => { ... }
    _ => { ... }
}
```

You will see this often with enums like `Reference`, `Message`, and `Movement`.

### `enum`

An enum is a type with a fixed set of named variants.

Examples in this repo:

- `Reference`
- `Message`
- `Movement`
- `AlignmentPath`

Enums are one of the main ways Rust encodes state and avoids invalid combinations.

### `struct`

A struct is a type with named fields.

Examples in this repo:

- `State`
- `Repository`
- `Settings`
- `App`

If an enum represents "which kind of thing is this?", a struct usually represents "what data does this thing hold?".

### `impl`

An `impl` block defines methods for a type.

Example shape:

```rust
impl State {
    pub fn new(...) -> Result<Self, TGVError> { ... }
}
```

This is how methods are attached to structs and enums in Rust.

### `pub`

`pub` means public. Without `pub`, an item is private to its module by default.

You will see:

- `pub struct`
- `pub enum`
- `pub fn`
- `pub mod`

If something is not visible where you expect it to be, check whether it is public.

### `use`

`use` imports names into the current file or module.

Example:

```rust
use gv_core::error::TGVError;
```

This is similar to imports in other languages.

### `?`

The `?` operator is a concise way to propagate errors upward.

Example:

```rust
let settings: Settings = cli.try_into()?;
```

This means:

- if the conversion succeeds, keep the value.
- if it fails, return the error from the current function immediately.

You will see `?` everywhere in this codebase.

### `Self`

`Self` refers to the type inside its own `impl` block.

Example:

```rust
impl Default for Settings {
    fn default() -> Self { ... }
}
```

That `Self` means `Settings`.

### borrowing and references

Rust distinguishes between owning a value and borrowing it.

- `String` owns a string.
- `&str` is a borrowed string slice.
- `&T` is an immutable reference.
- `&mut T` is a mutable reference.

You will often see function signatures like:

```rust
fn new(settings: &Settings) -> Result<Self, TGVError>
```

That means the function borrows `settings` instead of taking ownership of it.

### mutable variables

Variables are immutable by default in Rust.

If something needs to change, it must usually be marked with `mut`.

Example:

```rust
let mut app = App::new(settings).await?;
```

### traits

Traits are similar to interfaces in other languages. They define shared behavior.

Examples in this repo include service traits such as `TrackService`.

When you see:

```rust
impl TrackService for UcscDbTrackService
```

that means `UcscDbTrackService` provides the behavior required by the `TrackService` trait.

### async and `.await`

Rust uses `async` for asynchronous operations such as network access, file I/O, and database access.

Example shape:

```rust
pub async fn new(settings: Settings) -> Result<Self, TGVError>
```

If a function is `async`, calling it usually produces a future, and `.await` waits for the result:

```rust
let mut app = App::new(settings).await?;
```

This repo uses `tokio` as the async runtime.

### `derive`

Rust can auto-generate common trait implementations with `#[derive(...)]`.

Common examples:

- `Debug`
- `Clone`
- `PartialEq`
- `Eq`
- `Default`
- `Error`

Example:

```rust
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Reference { ... }
```

### `Cargo.toml`

This is Rust's project manifest file.

It defines:

- packages and workspace members.
- dependencies.
- compiler and build settings.
- sometimes feature flags and profiles.

In this repo:

- the workspace root `Cargo.toml` defines the workspace members and shared dependencies.
- each crate also has its own `Cargo.toml`.

### Cargo

Cargo is Rust's build tool and package manager.

You will use it for nearly everything:

- `cargo check`
- `cargo build`
- `cargo run`
- `cargo nextest run`
- `cargo test --doc`

For this repo, treat Cargo as the default entry point for development work.
