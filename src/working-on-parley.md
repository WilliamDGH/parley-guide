# Working on Parley

This chapter is the practical one: how to build, run and test, how snapshot tests work, and what a pull request is expected to look like. Everything here was checked against the repository at commit `2133d78` or run on this machine; the test counts are from a real run.

## Toolchain

- Rust 1.88 or later. CI tests the minimum supported version (MSRV) 1.88 for the published crates and the latest stable, 1.99.1 at the time of writing (`.github/workflows/ci.yml:6`). Bumping the MSRV is not a breaking change (`README.md`).
- Edition 2024 (`Cargo.toml:28`), so `if let ... && let ...` chains and `use<'a>` capture bounds appear throughout.
- No `unsafe` outside fontique, where it is used for platform bindings and memory-mapped font files (`Cargo.toml:76`, `fontique/src/source_cache.rs:252`).

## Build, run, test

```bash
cargo build --workspace
```

Run the renderers in `examples/` to see output as a PNG. Each writes a file in `examples/_output/` inside the repository:

```bash
cargo run -p swash_render
cargo run -p swash_render -- --tree
cargo run -p tiny_skia_render
cargo run -p vello_cpu_render
```

The interactive editor opens a window and needs a display:

```bash
cargo run -p editor
```

Run the tests:

```bash
cargo test --workspace
```

What passed on this machine on 25 October 2026, run as `cargo test -p parley_engine -p parley -p parley_tests`:

| suite | tests |
|---|---|
| `parley` unit tests (`parley/src/tests/`, plus module tests) | 105 |
| `parley_engine` unit tests | 33 |
| `parley_tests` integration tests | 249, 1 ignored |
| doc tests | 3 |

The first full build takes a few minutes because `parley_tests` pulls in `vello_cpu` for rendering. Later runs are fast.

The fonts used by all tests are in `parley_dev/assets/fonts` and are loaded explicitly; system fonts are disabled (`parley_tests/tests/util/env.rs:97`), so results are the same on every machine. The default test style stack is Roboto then Noto Kufi Arabic, with `LineHeight::FontSizeRelative(1.0)` kept for historical reasons (`env.rs:174`).

## Where tests live

| Kind | Where | How to add |
|---|---|---|
| Unit tests of private code | `#[cfg(test)] mod tests` at the bottom of each file, and `parley/src/tests/*.rs` for tests that need the crate's private items (`test_analysis.rs`, `test_builders.rs`, `test_style_metrics.rs`) | add a `#[test]` fn in the matching module |
| Engine tests | bottom of each `parley_engine/src/**/*.rs`; they embed Roboto and Noto Kufi with `include_bytes!` | same |
| Integration and snapshot tests | `parley_tests/tests/*.rs`, one file per topic (`basic`, `whitespace`, `wrap`, `lines`, `cursor`, `editor`, `emoji`, `content_widths`, `text_indent`, `styles/*`, ...) | put the test in the file for its topic, or `issues.rs` for a regression test named `issue_NNN`; name tests topic-first (`cursor_move_left`, not `move_left_cursor`) (`parley_tests/tests/mod.rs:16`) |
| Chrome line-break comparison | `parley_tests/tests/linebreaking_matches_chrome.rs` with recorded data under `parley_tests/linebreaking_browser_recorder/data` | regenerate with the browser recorder; see the module docs |
| Benchmarks | `parley_bench/src/benches.rs` | `cargo bench -p parley_bench` (tango-bench; see its README for comparing two builds) |

A typical integration test:

```rust
#[test]
fn plain_multiline_text() {
    let mut env = TestEnv::new(test_name!(), None);
    let text = "Hello world!\nLine 2\nLine 4";
    let builder = env.ranged_builder(text);
    let mut layout = builder.build(text);
    layout.break_all_lines(None);
    layout.align(Alignment::Start, AlignmentOptions::default());
    env.check_layout_snapshot(&layout);
}
```

`TestEnv` (`parley_tests/tests/util/env.rs:43`) owns a `FontContext` and `LayoutContext`, builds layouts with the default style stack, renders them with `vello_cpu`, and compares the PNG against `parley_tests/snapshots/<test_name>-<case>.png`. Snapshots must be under 32 KiB (`env.rs:153`). Many tests also assert numbers directly (`whitespace.rs`, `content_widths.rs`) and the project prefers that style where the intent can be read without opening an image (the `CursorTest` docs, `parley_tests/tests/util/cursor_test.rs:31`).

## Snapshot tests: the loop

1. Run the tests. A failing snapshot test writes the new image to `parley_tests/current/` and the failure message names the file.
2. Look at the difference. Either open both PNGs, or generate an HTML report:
   ```bash
   cargo xtask report
   ```
   or step through them interactively:
   ```bash
   cargo xtask review
   ```
   Both use [Kompari](https://github.com/linebender/kompari) (`xtask/src/main.rs`).
3. If the new image is right, accept it:
   ```bash
   PARLEY_TEST=accept cargo test -p parley_tests
   ```
   This overwrites the snapshots of the failing tests only (`env.rs:383`). `PARLEY_TEST=generate-all` writes every image to `current/` without changing snapshots.
4. Check for snapshots that no test uses any more:
   ```bash
   cargo xtask dead-snapshots
   ```
5. Commit the changed PNGs with the code. PNGs are optimised with `oxipng` on save (`env.rs:356`).

## Formatting and lints

CI runs these; run them before pushing:

```bash
cargo fmt --all --check
```

```bash
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

CI is stricter than that: it uses `cargo hack` to check every feature combination separately, including `no_std` on `x86_64-unknown-none` and `thumbv8m.main-none-eabihf`, 32-bit `powerpc`, `wasm32` and Android (`ci.yml:121` onwards). Writing `alloc::vec::Vec` instead of `std::vec::Vec` in library code and using `core::` paths is what keeps those green.

Other checks:

- Every `.rs` file starts with a copyright header (`.github/copyright.sh`), with the year the file was created:
  ```rust
  // Copyright 2026 the Parley Authors
  // SPDX-License-Identifier: Apache-2.0 OR MIT
  ```
- TOML is formatted with taplo (`.taplo.toml`): `taplo fmt --check --diff`.
- Each crate's `README.md` is generated from its `lib.rs` doc comment: after editing a crate doc, run `cargo rdme --workspace-project=<crate>` and commit the README (`parley/src/lib.rs:5`).
- Spelling is checked with `typos` (`.typos.toml`).
- Formatting follows the Linebender guidelines (`https://linebender.org/wiki/formatting-scheme/`, applied in `#914`), and the lint set is the Linebender canonical one (`Cargo.toml:75`).

## How a pull request looks

The template is `.github/pull_request_template.md`. Three things are expected:

1. **An `LLM Contributions:` line.** Linebender has a policy on LLM use (`https://linebender.org/wiki/llm-policy/`). State what, if anything, was generated, or write `None`. Descriptions written by an LLM are not allowed; write your own.
2. **A `**Changelog**` block.** A quote block with the entry you propose for `CHANGELOG.md`, under `Added`, `Changed`, `Fixed` or `Removed`, written for users. Maintainers edit it and copy it into the changelog at release time; do not edit `CHANGELOG.md` yourself (`CHANGELOG.md:7`). Write `**Changelog: None**` for internal changes.
3. **Small, focused PRs.** The history is almost entirely single-purpose PRs with titles that name the change and sometimes the crate: "fontique: Fix synthesized `slnt` sign (#922)", "Don't letter-space for zero-width spaces (plus some others) (#924)". Behaviour changes come with a test, usually in `parley_tests`, and often a benchmark note when they touch the hot loops.

Conventions visible in the code that reviewers will expect you to follow:

- Doc comments on public items; `missing_docs` is a warning that `parley` currently allows crate-wide but new code is documented.
- Comments cite the spec: "CSS Text 4 § 4.3.2", "UAX #14 LB6", with a link where one exists.
- Casts from `usize` to `u32`/`u16` carry `#[expect(clippy::cast_possible_truncation, reason = "...")]`.
- Tests that pin down a deviation or a bug in a dependency say so and say when they can be removed (`analysis.rs:1087`).
- `TODO:` comments are fine and common; use them to mark known gaps rather than silently leaving them.

Discussion happens on the Linebender Zulip, `#parley` channel; it is readable without an account.

## Debugging tips

- `Layout` implements `Debug`. `{:?}` prints a summary (lines, runs, styles), `{:#?}` dumps all of `LayoutData` (`layout.rs:29`). The dump is long but shows every index.
- The probe programs in this book's `scratch/probe` folder print analysis, shaped text and layout for any string; copy and change them.
- `parley_tests/lines.html` renders text in a browser with the same fonts and settings as the `lines.rs` tests, for comparing line heights with Chrome.
- When a snapshot changes unexpectedly, `cargo xtask report` shows a diff image; one-pixel baseline shifts usually mean a quantization change in `style_metrics.rs` or `finish_line`.
- Set `max_advance` to `None` to rule out line breaking, and `quantize` to `false` to see exact metrics.
