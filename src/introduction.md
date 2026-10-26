# Introduction

This is a guide to the code of [Parley](https://github.com/linebender/parley), the rich text layout library from the Linebender project. It explains how Parley works end to end, what the main data structures are, how they point at each other, and where in the code to look for each piece. It is written for someone who knows web front-end work well, is new to Rust, and is about to make their first contributions to Parley.

It is not reference documentation. The reference is the rustdoc on [docs.rs](https://docs.rs/parley/0.12.0/parley/), and this guide links to it instead of repeating it.

## Parley in a nutshell

You give Parley a string and a set of styles (fonts, sizes, colors, spacing, line heights). Parley finds a font for every character, asks [HarfRust](https://github.com/harfbuzz/harfrust) to shape the characters into glyphs, breaks the glyphs into lines that fit a given width, puts right-to-left text in visual order, aligns the lines, and gives you back positioned glyph runs that a renderer can paint. It also answers editing questions: where is the caret for byte 13, which character is under this point, what rectangles cover this selection.

Parley is a library, not a widget. It does not paint pixels (that is a renderer such as Vello, tiny-skia or Swash), it does not store documents, and it does not flow one paragraph into the next. One `Layout` is one block of inline content, like one `<p>` element in CSS, including its `<br>` forced breaks.

## How to read this guide

The chapters are in the order a newcomer should read them. Each one takes about fifteen minutes.

| Chapter | What you will be able to do afterwards |
|---|---|
| [Overview and mental model](overview.md) | Name the crates, draw the pipeline, say what Parley is and is not. |
| [One string, start to finish](walkthrough.md) | Follow one real string through every stage and re-run it yourself. |
| [Data flow on one page](data-flow.md) | See the input, the data after every stage and the output on one page, and follow one character through every index. |
| [Architecture map](architecture.md) | Find the module that owns a feature, and know what it must not know about. |
| [Core data structures](data-structures.md) | Read any index in the code and know which array it points into. |
| [Key algorithms](algorithms.md) | Explain line breaking, hanging whitespace, justification, bidi reordering and the line box with numbers. |
| [Cross-cutting rules and invariants](invariants.md) | Avoid the mistakes that break the whole pipeline, and map every style property to its CSS spec section. |
| [Glossary](glossary.md) | Look up any term, with a CSS or TypeScript analogy. |
| [Working on Parley](working-on-parley.md) | Build, test, regenerate snapshots and open a PR the way the maintainers expect. |
| [Further reading](further-reading.md) | Go deeper into text layout and the decisions behind Parley. |

If you only have one hour, read the Overview, then the Walkthrough, then skim the Glossary.

## Conventions

- **Code locations** are written as `path:line`, relative to the repository root. All line numbers refer to commit `2133d78` of `main` (22 October 2026), which is 27 commits after the 0.12.0 release commit (`#888`, 9 October 2026). Lines drift as the code changes; the function names next to them should still let you find the spot. To open a location on GitHub, use `https://github.com/linebender/parley/blob/2133d78a55de40f0d93a303240ba017413855663/<path>#L<line>`.
- **Version**: Parley 0.12.0, released 9 October 2026, plus the unreleased commits on `main` up to `2133d78`. The biggest unreleased change is that in-flow inline boxes are now represented by a U+FFFC character in the text (`#902`).
- **Numbers** in this guide were printed by small programs that are in the `scratch/probe` folder next to this book. Nothing was typed in by hand. Where a claim could not be checked by running code or reading it, the text says **unverified**.
- **Terms** are defined on first use and collected in the [Glossary](glossary.md). When a term has a CSS equivalent, the CSS name is given too.
- Examples avoid look-alike characters (such as typographic quotes or the multiplication sign) so that you can type them.

## Re-running the examples

The probe programs live in `scratch/probe` beside this book's `src` folder. They depend on the Parley crates by relative path (`../../../parley/<crate>` from `scratch/probe/`), that is on the checkout at `/Users/guanhuading/personal/parley`, and use whatever is checked out there, including a feature branch with uncommitted changes. Edit the paths in `scratch/probe/Cargo.toml` if your checkout lives somewhere else.

```bash
cd scratch/probe && cargo run --bin walkthrough -- 80
```

The argument is the maximum line width. The second program prints the whitespace, justification, spacing and line height experiments used in the algorithms chapter:

```bash
cd scratch/probe && cargo run --bin spacing
```
