# Further reading

## Parley's own documents

- rustdoc: [parley](https://docs.rs/parley/0.12.0/parley/), [parley_engine](https://docs.rs/parley_engine/0.12.0/parley_engine/), [fontique](https://docs.rs/fontique/0.12.0/fontique/), [parlance](https://docs.rs/parlance/0.1.1/parlance/). The module docs of `parley/src/layout/style_metrics.rs` (aligned subtrees) and `parley/src/layout/whitespace.rs` (conditional hanging) are the two best pieces of prose in the codebase.
- `CHANGELOG.md`, especially the 0.12.0 highlights. Every entry links its PR; the PR discussions are where the "why" lives.
- `doc/concept.md` and `doc/design.md` from 2021: the original pipeline description with a worked example.
- `parley_tests/README.md`: the snapshot test workflow.
- The [Parley WPT guide](https://github.com/DioxusLabs/blitz/blob/main/docs/parley.md) in Blitz, for running the Web Platform Tests against Parley.

## Pull requests that explain big decisions

| Topic | PR |
|---|---|
| Why swash was replaced by HarfRust for shaping | `#400` |
| Why text analysis moved to ICU4X and how `parley_data` bakes tables | `#436` |
| Why `parley_engine` exists and what it is for | `#388`, `#705`, `#679` |
| Shaped clusters versus graphemes versus atoms | `#715` |
| Why spacing and justification are applied on read | `#738` |
| Line box sizing following CSS 2.2 section 10.8 | `#697` |
| Conditional versus unconditional hanging whitespace, with an HTML sample | `#762`, `#790`, `#831` |
| `vertical-align` and aligned subtrees | `#766` |
| Floats, exclusions and the low-level `BreakLines` API | `#421` |
| Matching Chromium's line breaks | PRs `#485`, `#640`; issue `#619` |
| Inline boxes as U+FFFC characters | `#902` |
| The Han fallback workaround for common-script punctuation | issue `#597` |
| `white-space-collapse` as a style and whitespace collapsing across spans | `#786`, `#814` |

Open PRs at `https://github.com/linebender/parley/pull/<number>` and issues at `https://github.com/linebender/parley/issues/<number>`.

## Text layout in general

- [HarfBuzz manual](https://harfbuzz.github.io/): "What is text shaping?", "Clusters", and "Working with HarfBuzz clusters". Parley uses the monotone-characters cluster level the last chapter describes.
- Alexis Beingessner, [Text Rendering Hates You](https://faultlore.com/blah/text-hates-you/): why every stage above is harder than it looks.
- Raph Levien's writing on text, including the Xi editor's [Rope science](https://xi-editor.io/docs/rope_science_00.html) series and the [Linebender wiki](https://linebender.org/wiki/) pages on text.
- [CSS Text Module Level 3](https://www.w3.org/TR/css-text-3/) and [Level 4](https://www.w3.org/TR/css-text-4/): white space processing (section 4), line breaking (section 5), alignment (section 7), spacing (section 8). Parley's comments cite these sections.
- [CSS Inline Layout Module Level 3](https://www.w3.org/TR/css-inline-3/): the line box, inline box height and aligned subtrees. Plus the older but still normative [CSS 2.2 section 10.8](https://www.w3.org/TR/CSS22/visudet.html#line-height).
- [CSS Fonts Module Level 4](https://www.w3.org/TR/css-fonts-4/): the font matching algorithm fontique implements, and the first available font.
- Unicode: [UAX 9 Bidirectional Algorithm](https://www.unicode.org/reports/tr9/), [UAX 14 Line Breaking](https://www.unicode.org/reports/tr14/), [UAX 29 Text Segmentation](https://www.unicode.org/reports/tr29/), [UTS 51 Emoji](https://www.unicode.org/reports/tr51/), [UAX 24 Script Property](https://www.unicode.org/reports/tr24/) (section 5 is the heuristic Parley's itemizer uses for Common characters).
- [ICU4X](https://github.com/unicode-org/icu4x) docs for `icu_segmenter`, `icu_properties` and `icu_normalizer`.
- [Fontations](https://github.com/googlefonts/fontations) (`read-fonts`, `skrifa`) and [HarfRust](https://github.com/harfbuzz/harfrust).
- Blink's inline layout for comparison: the `LayoutNG` inline design notes in the Chromium source tree, and the files Parley's override table was copied from: `character_property_data_generator.cc` (linked from `parley_engine/src/break_overrides.rs:44`) and `text_break_iterator.cc` (linked from `break_overrides.rs:76` and `:86`).

## Rust, for a web engineer

- [The Rust Book](https://doc.rust-lang.org/book/), chapters 4 (ownership), 8 (collections), 10 (generics and lifetimes) and 13 (iterators) cover almost everything Parley's code uses.
- [Rust for TypeScript developers](https://github.com/rust-lang/rustlings) exercises, for the syntax.
- The `no_std` chapter of the [Embedonomicon](https://docs.rust-embedded.org/embedonomicon/) explains why Parley imports `alloc::vec::Vec`.

## How this guide was written

The style follows [rust-analyzer's architecture document](https://github.com/rust-lang/rust-analyzer/blob/master/docs/book/src/contributing/architecture.md) and matklad's [ARCHITECTURE.md](https://matklad.github.io/2021/02/06/ARCHITECTURE.md.html) post for the map; [Diátaxis](https://diataxis.fr) for keeping explanation, walkthrough and how-to apart; the [rustc dev guide](https://rustc-dev-guide.rust-lang.org) for telling the story in pipeline order; and the [Tokio tutorial](https://tokio.rs/tokio/tutorial) for small steps with real output.
