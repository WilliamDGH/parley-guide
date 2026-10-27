# Cross-cutting rules and invariants

This chapter collects the rules that hold across the whole pipeline. Break one of them and a test far away from your change will fail, usually in `parley_tests`. After it you also have a table mapping each `StyleProperty` to its CSS property and spec section, so that you can check Parley's behaviour against the standard.

## Shaped advances are independent of layout

The glyph advances in `ShapedText` are what HarfRust produced and nothing else. Letter spacing, word spacing and justification are added *when the data is read*, by `EffectiveSpacing` (`parley/src/layout/spacing.rs:167`), which combines the run's `Spacing` with the line's `Justification`. Every reader goes through it:

- the line breaker measures atoms with `EffectiveSpacing::new(spacing, Justification::NONE)` (`line_break.rs:1144`);
- `Run::advance`, `Cluster::advance` and the glyph iterators use `Run::line_spacing()` (`run.rs:129`), which includes the line's justification when the run is scoped to a line;
- `ContentWidthsMeasurer` adds the gaps itself, atom by atom (`data.rs:612`).

**Why**: before `#738` (August 2026) justification mutated advances in place, so aligning twice, or breaking after aligning, gave wrong widths. Keeping shaping output immutable makes repeated calls to `break_all_lines` and `align` safe, and lets one `ShapedText` be shared by every line. Order still matters: line breaking resets offsets and justification, so call `align` after the latest `break_all_lines` (`layout/layout.rs:233`).

**How to apply**: never write to `ShapedCluster::advance` or `Glyph::advance` outside `parley_engine`. The one exception is `set_inline_object_advance` (`shaped_text.rs:234`), which fills in the width of an inline object whose cluster was shaped with advance 0. If you need a new kind of extra space, add it to `Gaps` and `EffectiveSpacing`.

## Per-layout data versus per-line data

Some facts are true for the whole layout and some only for one line. Mixing them up is the second most common bug.

| Per layout (written by build) | Per line (written by break or align) |
|---|---|
| `ShapedText`, `runs: Vec<RunData>`, `items` | `lines`, `line_items`, `aligned_subtree_offsets` |
| `styles`, `style_metrics` (span boxes, `baseline_offset` within a subtree) | `LineData::aligned_subtree_offsets` (where each `top`/`bottom` subtree sits on *this* line) |
| `RunData::spacing` (letter and word spacing) | `LineData::justification` (the stretch for *this* line) |
| `RunData::line_height`, `run_box` | `LineMetrics` (baseline, line box, content box, advance, hanging) |
| `inline_boxes` with `baseline_offset` | `LineMetrics::offset` (alignment) |

A `Run` view carries `Option<&LineItemData>` to say which world it is in (`run.rs:29`). With `None` it is the whole shaped run and justification is `NONE`; with `Some` it is narrowed to the line's atom-aligned cluster range and the line's justification applies (`run.rs:129`). A style's baseline shift is therefore computed in two steps: `StyleMetrics::baseline_offset` within its aligned subtree (per layout) plus `LineData::aligned_subtree_offset` (per line), combined in `LineData::style_baseline` (`data.rs:136`).

## Style indices

- The style table is `Vec<Style>` on the layout and `Vec<ResolvedStyle>` during build. Index 0 is the root style, and the root is its own parent (`layout/mod.rs:51`).
- The table is parent-first: `styles[i].parent < i` for `i > 0`, guaranteed by both builders (`resolve/range.rs:148`, `resolve/tree.rs:215`) and relied on by `resolve_style_metrics` (`style_metrics.rs:103`).
- A character's style is `Character::style_index`. A shaped cluster's style is that of its logically first character (`shaped_text.rs:559`). A `Cluster`'s (grapheme's) style is that of its first character too (`cluster.rs:177`). A `GlyphRun` has one style and ends where the first-character style changes between atoms (`line.rs:482`).
- A ligature can cross a style boundary ("ffi" with a different brush on the second `f`). Then the atom has mixed styles: `RunData::has_mixed_style_atoms` (`data.rs:44`) is set and the line box collects the span boxes of every style in the atom (`line_break.rs:448`). The glyphs are drawn with the first character's style. A font or synthesis change splits the shaping run; making the second `f` bold in the guide's Roboto setup prevents the `ffi` ligature.
- Style indices are `u16`. Both builders `assert!` when the table would exceed `u16::MAX` entries.

## Atom alignment

Everything that slices shaped text does so at atom boundaries:

- line boundaries: `LineState::clusters` and `LineItemData::shaped_cluster_range` are atom-aligned (`line_break.rs:66`, `data.rs:160`);
- glyph runs: `GlyphRun::shaped_clusters` (`line.rs:289`);
- `ShapedSlice::narrow` and `atoms_from` `debug_assert` it (`atom.rs:282`, `atom.rs:196`).

Consequence: a line always begins and ends on a grapheme boundary, so a caret can always be placed at a line start or end.

## Analysis and shaping see the same text

`Analyzer::analyze` and `Shaper::shape_text` must be given the identical `&str`. The engine does not compare their contents: different text with the same character count can pass without a panic. The panic at `itemize.rs:207` catches the text iterator ending before the analysis data does. The `Item`s passed to `shape_text` must tile the character count exactly (`shaper.rs:176`, `shaper.rs:223`). `char_style_indices` must have one entry per character (`shaper.rs:166`).

Empty text is special-cased: the builder substitutes a single space so that the layout has metrics to size a caret (`parley/src/analysis.rs:16`, `parley/src/shape/mod.rs:58`), then `BreakLines::finish` resets the first line's text range to `0..0` (`line_break.rs:1468`).

## `ShapedText` covers the text, unless fonts are missing

Normally `ShapedText::characters().len()` equals the character count, and every character is in exactly one run (`shaper.rs:227`). If the `FontSelector` returns `None`, shaping stops and the result covers a prefix of the text (`shaper.rs:203`). `parley` tries to avoid this by falling back through the sans-serif, serif and monospace generic families (`any_font`, `parley/src/shape/mod.rs:447`). This does not scan every registered font: if neither the requested families nor the fallback mappings yield a font, a `Layout` can be partial even when the `FontContext` contains fonts. Code that walks runs should not assume it reached the end of the text.

## Quantize and scale

- `scale` is applied once, at style resolution, to every length: font size, spacing, decoration offsets, absolute line heights, `vertical-align` lengths (`resolve/mod.rs:136`). Inline box sizes are not scaled; callers pass them in layout units.
- `quantize` rounds the font's ascent, descent and leading before summing them for `line-height: normal` (`style/mod.rs:96`), rounds span ascents and descents, floors the half-leading above the baseline, and rounds baseline offsets (`style_metrics.rs:165`, `:228`). It also rounds line tops and bottoms (`line_break.rs:1619`) and inline box heights for `top`/`bottom` boxes. It never rounds glyph advances or x positions. Fractional inline box dimensions can leave fractional baselines, and fractional line heights can make rounded line boxes overlap or leave gaps (`line_break.rs:1626`). The `ranged_builder` docs list vertical coordinates to round before painting (`context.rs:87`); x positions can stay fractional, as in the Swash renderer (`examples/swash_render/src/main.rs:324`).

## Bidi facts to remember

- `bidi_levels()` is empty when bidi resolution is skipped; `Itemizer` treats that as "every character at the paragraph level" (`itemize.rs:171`). Left-to-right text can still need resolution if it contains embedding, override or isolate initiators, or Arabic numbers (`bidi.rs:716`).
- Levels are per character; runs never span a level change.
- Items on a line are reordered at commit time, so `line_items` are visual and `items` are logical.
- Trailing whitespace hangs only when its run has the paragraph level, because UAX 9 L1 is not implemented (`line_break.rs:1908`, `data.rs:516`). Marked as a known limitation in the code.

## Whitespace classes

The `Whitespace` enum (`parley_engine/src/shape/cluster.rs:81`) classifies every Unicode `White_Space` character into one of seven classes, and `from_char` maps nothing else (a test checks this against the whole range, `cluster.rs:636`):

| class | characters | hangs? | word separator (justified, word-spaced)? | collapsible under `collapse`? |
|---|---|---|---|---|
| `Space` | U+0020 | yes (see rules) | yes | yes |
| `NoBreakSpace` | U+00A0 | no | yes | no |
| `IdeographicSpace` | U+3000 | yes | no | no |
| `OtherSpaceSeparator` | U+1680, U+2000..U+200A, U+202F, U+205F | no | no | no |
| `Tab` | U+0009 | yes | no | yes |
| `Newline` | U+000A, U+000D, U+2028, U+2029 | yes (zero advance) | no | LF and CR yes, LS and PS no |
| `ControlWhitespace` | U+000B, U+000C, U+0085 | no | no | FF yes (ASCII whitespace); VT and NEL no |

"Collapsible" is `WhiteSpaceCollapse::is_collapsible` (`whitespace.rs:39`): ASCII whitespace under `Collapse`, only space and tab under `PreserveBreaks`. CSS Text 4's hanging rules include U+3000 and other space separators. Parley hangs U+3000 (`#784`), but leaves the `OtherSpaceSeparator` class out (`whitespace.rs:56`).

## Where CSS semantics live

Parley follows CSS where CSS has an answer. This table maps every `StyleProperty` (`parley/src/style/mod.rs:227`) to its CSS property, the spec section Parley follows, and where the behaviour is implemented.

| `StyleProperty` | CSS property | Spec | Implemented in |
|---|---|---|---|
| `FontFamily` | `font-family` | CSS Fonts 4 section 2.1; first available font, section 3.4 | `resolve/mod.rs:217`, `shape/mod.rs:303`, `style_metrics.rs:359` |
| `FontSize` | `font-size` | | `shape/mod.rs` items; scaled in `resolve/mod.rs:145` |
| `FontWidth`, `FontStyle`, `FontWeight` | `font-stretch`, `font-style`, `font-weight` | CSS Fonts 4 section 5 (matching algorithm, oblique threshold 14deg instead of 20deg) | `fontique/src/matching.rs:31` |
| `FontVariations` | `font-variation-settings` | CSS Fonts 4 section 2.9 | `resolve/mod.rs:273`, `shaper.rs:462` |
| `FontFeatures` | `font-feature-settings` | CSS Fonts 4 section 6.12 | `resolve/mod.rs:298`, `shaper.rs:278` |
| `Locale` | `lang` attribute | affects shaping (HarfRust), fallback fonts, and CJK line breaking (`line-break` normal and loose) | `shaper.rs:273`, `analysis.rs:306` |
| `Brush`, `UnderlineBrush`, `StrikethroughBrush` | `color`, `text-decoration-color` | | passed through to `layout::Style` |
| `Underline`, `UnderlineOffset`, `UnderlineSize`, `Strikethrough`, `StrikethroughOffset`, `StrikethroughSize` | `text-decoration-line`, `text-underline-offset`, `text-decoration-thickness` | CSS Text Decoration 4; metrics from the font, HarfBuzz fallbacks when missing | `layout/mod.rs:79`, `shaped_text.rs:105` |
| `LineHeight` | `line-height` | CSS 2.2 section 10.8, CSS Inline 3 section 4.1 | `style/mod.rs:49`, `style_metrics.rs:228`, `data.rs:56` |
| `VerticalAlign` | `vertical-align`, `alignment-baseline`, `baseline-shift` | CSS 2.2 section 10.8.1, CSS Inline 3 section 4.2 | `style/mod.rs:163`, `style_metrics.rs:287`, `line_break.rs:1546` |
| `WordSpacing` | `word-spacing` | CSS Text 4 section 8.1 | `spacing.rs:73` |
| `LetterSpacing` | `letter-spacing` | CSS Text 4 section 8.2 (applied after each typographic character unit); CSS Text 3 section 7.2 (disable optional ligatures) | `spacing.rs:73`, `shape/mod.rs:37` |
| `WordBreak` | `word-break` | CSS Text 3 section 5.2 | `analysis.rs:301` via ICU `LineBreakWordOption` |
| `LineBreak` | `line-break` | CSS Text 3 section 5.3; `anywhere` adds opportunities around every grapheme | `analysis.rs:294`, `analysis.rs:862` |
| `OverflowWrap` | `overflow-wrap` | CSS Text 3 section 5.5 (emergency breaks) | `line_break.rs:1219` |
| `TextWrapMode` | `text-wrap-mode` | CSS Text 4 section 5.1 | `line_break.rs:1210` |
| `WhiteSpaceCollapse` | `white-space-collapse` | CSS Text 4 section 4.1 (collapsing, `TreeBuilder` only), section 4.3.2 (hanging), `break-spaces` | `resolve/tree.rs:108`, `whitespace.rs:61`, `analysis.rs:830` |

Layout-level options that are not styles:

| API | CSS | Spec | Implemented in |
|---|---|---|---|
| `Layout::align(Alignment, AlignmentOptions)` | `text-align`, `text-align-last` | CSS Text 3 section 7.1 to 7.3 | `alignment.rs:69` |
| `Layout::set_text_indent` | `text-indent` with `hanging` and `each-line` | CSS Text 3 section 6.1 | `layout/layout.rs:217`, `IndentOptions` at `layout/mod.rs:103`, `line_break.rs:1478` |
| `InlineBox` | atomic inline | CSS Display 3, CSS Text 4 section 5.4 (`atomic-compat-wrap`, not yet followed) | `builder.rs:447` |
| `InlineBoxKind::OutOfFlow` | `position: absolute` | | `shape/mod.rs:194` |
| `InlineBoxKind::CustomOutOfFlow` | `float` (caller places it) | | `line_break.rs:1125` |
| `BaseDirection` | `direction`, `unicode-bidi: plaintext` for `Auto` | UAX 9 P2 and P3 | `bidi.rs:71` |
| `set_line_break_override(CHROMIUM_LINE_BREAK_OVERRIDE)` | none; matches Blink's behaviour | | `break_overrides.rs:56` |

Where Parley knowingly deviates from CSS or the Unicode algorithms, the code says so in a comment. The ones found while writing this guide:

- Not hanging the `OtherSpaceSeparator` class, although CSS includes those characters in its hanging rules (`whitespace.rs:56`).
- Letter spacing between cursive letters is applied although CSS Text 4 section 8.2.1 says not to (`data.rs:30`).
- Only space and no-break space are word separators; CSS also lists the Ethiopic word space and others (`spacing.rs:14`).
- `break-spaces` overrides UAX 14 LB7 (`analyzer.rs:93`).
- A Han fallback family is always added to the query (`query.rs:105`).
- UAX 9 L1 is not applied to trailing whitespace (`line_break.rs:1908`).
- Grapheme segmentation restarts at item boundaries, so a flag emoji split by a style change becomes two graphemes (`shaped_text.rs:395`, `shaped_text.rs:918`).
