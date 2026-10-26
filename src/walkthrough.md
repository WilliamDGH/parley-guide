# One string, start to finish

This chapter follows one string through every stage of the pipeline and shows the real numbers Parley produced. After it you will know what each stage adds, which indices appear where, and how to print the same data for any string of your own.

The string is `"Hi مرحبا office!\nBye"`. It has Latin text, an Arabic word (right-to-left), a word with the `ffi` ligature, a forced newline, and a font size change: `Hi` is 24px and everything else is 16px. The fonts are Roboto (Latin) and Noto Kufi Arabic from `parley_dev/assets/fonts`, the same fonts the test suite uses.

Everything below was printed by `scratch/probe/src/bin/walkthrough.rs`. Run it with a width argument:

```bash
cd scratch/probe && cargo run --bin walkthrough -- 80
```

## Stage 0: the input

```text
TEXT = "Hi مرحبا office!\nBye"  (25 bytes, 20 chars)
```

Two index spaces appear immediately. Rust strings are UTF-8, so the five Arabic letters take two bytes each: the text is 25 bytes but 20 characters. Parley's public API uses **byte** indices (`text_range()`, `Cursor::index()`), like `str` slicing in Rust. The engine's shaping data uses **character** indices (one per Unicode scalar value, which is one Rust `char`). You will convert between them constantly; the [data structures chapter](data-structures.md) has the table.

The styles for the layout are set up like this (`walkthrough.rs`, "layout" section):

```rust
let mut builder = lcx.ranged_builder(&mut fcx, TEXT, 1.0, true);
builder.push_default(StyleProperty::FontFamily(FontFamily::List(FAMILIES.into())));
builder.push_default(StyleProperty::FontSize(16.0));
builder.push(StyleProperty::FontSize(24.0), 0..2);
```

`FAMILIES` is `["Roboto", "Noto Kufi Arabic"]`, a font stack like CSS `font-family: Roboto, "Noto Kufi Arabic"`.

## Stage 1: style resolution

`RangedBuilder::build` (`parley/src/builder.rs:127`) first turns the ranged properties into a **style table** and **style runs**. The table holds every distinct resolved style; the runs are non-overlapping byte ranges, each pointing at a table index. For our text:

| Style index | Font size | Covers |
|---|---|---|
| 0 (root) | 16 | bytes 2..25 |
| 1 | 24 | bytes 0..2 |

The probe confirms `styles in table: 2`. The splitting is done by `RangedStyleBuilder::finish` (`parley/src/resolve/range.rs:75`): it starts with one span covering the whole text in the root style, then for every pushed property splits the spans at the property's range and applies it, and finally merges neighbours that ended up equal. The root style is always index 0 and is the parent of every other style (`parley/src/resolve/range.rs:148`).

"Resolved" means two things (`ResolveContext::resolve_property`, `parley/src/resolve/mod.rs:136`): lengths are multiplied by the display scale, and font family names are looked up in the `FontContext` and replaced by a handle to a cached list of `FamilyId`s. From here on nobody compares strings.

The builder also computes `char_style_indices` (`parley/src/builder.rs:482`): one `u16` per character, the style index for that character. This is the only form in which styles reach `parley_engine`.

## Stage 2: analysis

Analysis is font-independent. `Analyzer::analyze` (`parley_engine/src/analyzer.rs:42`) walks the string once and records, for every character, a `CharInfo` (`parley_engine/src/analysis.rs:322`) with its script and a bag of flags, plus a bidi level when the paragraph is not purely left-to-right. The probe printed:

```text
paragraph_level = BidiLevel(0)
char byte text   scr    bidi grapheme  wrap word
   0    0 H      Latn      0     true false true
   1    1 i      Latn      0     true false false
   2    2 space  Zyyy      0     true false true
   3    3 م      Arab      1     true  true true
   4    5 ر      Arab      1     true false false
   5    7 ح      Arab      1     true false false
   6    9 ب      Arab      1     true false false
   7   11 ا      Arab      1     true false false
   8   13 space  Zyyy      0     true false true
   9   14 o      Latn      0     true  true true
  10   15 f      Latn      0     true false false
  11   16 f      Latn      0     true false false
  12   17 i      Latn      0     true false false
  13   18 c      Latn      0     true false false
  14   19 e      Latn      0     true false false
  15   20 !      Zyyy      0     true false true
  16   21 \n     Zyyy      0     true false true
  17   22 B      Latn      0     true false true
  18   23 y      Latn      0     true false false
  19   24 e      Latn      0     true false false
```

Reading the columns:

- **scr** is the Unicode script. `Zyyy` is "Common": spaces and punctuation have no script of their own. Itemization will later give them the script of their neighbours.
- **bidi** is the embedding level from the Unicode Bidirectional Algorithm (UAX 9), computed by `BidiResolver::resolve` (`parley_engine/src/bidi.rs:52`). Even levels are left-to-right, odd levels right-to-left. The paragraph level is 0 because the first strong character, `H`, is Latin. The Arabic letters get level 1. Both spaces stay at 0.
- **grapheme** is `is_grapheme_start`. Every character here starts its own grapheme cluster (UAX 29). A combining accent would show `false`.
- **wrap** is `is_soft_wrap_opportunity`: may a line start before this character? Yes before `م` and before `o`, which are the characters after a space (UAX 14 puts the opportunity after the space, so Parley stores it on the next character). There is no opportunity before `B`: the newline is a *mandatory* break and is handled separately (`parley_engine/src/analysis.rs:856`).
- **word** is `is_word_boundary` (UAX 29 word boundaries), used by double-click selection and ctrl+arrow.

The segmenters come from ICU4X (`icu_segmenter`), the per-character properties from a baked table in `parley_data` so that one lookup gives script, general category, bidi class and emoji flags together (`parley_data/src/lib.rs:53`).

## Stage 3: itemization

Shaping works on runs of text that share a direction, a script, and the shaping-relevant style options (font size, language, features, variations). Two things cut the text into such runs.

First, `parley` builds **items** at style changes that matter for shaping (`parley/src/shape/mod.rs:151`): font size, locale, variations, features, letter spacing, word spacing. For our text that is one cut after `Hi`, giving two items: chars `0..2` at 24px and `2..20` at 16px.

Second, inside each item the engine's `Itemizer::next` (`parley_engine/src/itemize.rs:150`) further splits into **segments** wherever the bidi level or the script changes. Common-script characters inherit the script of the preceding character (`parley_engine/src/itemize.rs:178`). The result for our text is five segments:

| Segment | chars | script | bidi | why it starts |
|---|---|---|---|---|
| 1 | 0..2 `Hi` | Latn | 0 | start, item 1 |
| 2 | 2..3 ` ` | Latn | 0 | item 2 starts (font size 16) |
| 3 | 3..8 `مرحبا` | Arab | 1 | bidi level and script change |
| 4 | 8..9 ` ` | Arab | 0 | bidi level drops back to 0; script is still "Arab" by inheritance |
| 5 | 9..20 `office!\nBye` | Latn | 0 | script changes to Latin |

Segment 4 is worth a second look: the space after the Arabic word inherits the Arabic script, but has level 0. This matters for font selection in a moment.

## Stage 4: font selection and shaping

For each segment, `shape_segment` (`parley_engine/src/shape/shaper.rs:239`) walks the graphemes, asks a `FontSelector` (`parley_engine/src/shape/shaper.rs:485`) for a font for each one, and groups consecutive graphemes that got the same font into a **shaped run**. Each run is sent to HarfRust as one buffer and comes back as glyphs.

Parley's selector (`parley/src/shape/mod.rs:303`) asks fontique for the style's family list, in order, followed by script-based fallback families, and picks the first font whose character map covers the whole grapheme (`calculate_coverage`, `parley_engine/src/shape/cluster.rs:353`). Roboto covers Latin and the space; it has no Arabic, so the Arabic graphemes fall through to Noto Kufi Arabic.

The probe shapes the text with a deliberately simpler selector (Arabic script gets Noto Kufi, everything else gets Roboto), which makes one difference visible. Here is the shaped output, with the engine's indices:

```text
fonts: 2
characters: 20  shaped_clusters: 18  glyphs (stored): 2
run 0: bytes 0..2   chars 0..2  clusters 0..2  glyphs 0..0 font#0 size 24 bidi 0 script Latn ascent 22.27 descent 5.86
   cluster 0: chars 0..1 H     advance 17.11 glyph_len 1 inline true  glyphs [44@17.11]
   cluster 1: chars 1..2 i     advance  5.82 glyph_len 1 inline true  glyphs [77@5.82]
run 1: bytes 2..3   chars 2..3  clusters 2..3  glyphs 0..0 font#0 size 16 bidi 0 script Latn ascent 14.84 descent 3.91
   cluster 2: chars 2..3 space advance  3.96 glyph_len 1 inline true  glyphs [4@3.96]
run 2: bytes 3..13  chars 3..8  clusters 3..8  glyphs 0..2 font#1 size 16 bidi 1 script Arab ascent 20.51 descent 9.84
   cluster 3: chars 3..4 م     advance  9.18 glyph_len 1 inline true  safe_break true   glyphs [84@9.18]
   cluster 4: chars 4..5 ر     advance  5.74 glyph_len 1 inline true  safe_break false  glyphs [30@5.74]
   cluster 5: chars 5..6 ح     advance  9.28 glyph_len 1 inline true  safe_break true   glyphs [26@9.28]
   cluster 6: chars 6..7 ب     advance  5.46 glyph_len 2 inline false safe_break false  glyphs [309@0.00, 15@5.46]
   cluster 7: chars 7..8 ا     advance  5.06 glyph_len 1 inline true  safe_break false  glyphs [9@5.06]
run 3: bytes 13..14 chars 8..9  clusters 8..9  glyphs 2..2 font#1 size 16 bidi 0 script Arab
   cluster 8: chars 8..9 space advance  5.44 glyph_len 1 inline true  glyphs [3@5.44]
run 4: bytes 14..25 chars 9..20 clusters 9..18 glyphs 2..2 font#0 size 16 bidi 0 script Latn
   cluster 9:  chars 9..10  o     advance  9.12 glyph_len 1 inline true  glyphs [83@9.12]
   cluster 10: chars 10..13 ffi   advance 13.66 glyph_len 1 inline true  glyphs [446@13.66]
   cluster 11: chars 13..14 c     advance  8.38 ...
   cluster 12: chars 14..15 e     advance  8.48 ...
   cluster 13: chars 15..16 !     advance  4.12 ...
   cluster 14: chars 16..17 \n    advance  0.00 glyph_len 0 inline false ws Newline glyphs []
   cluster 15: chars 17..18 B     advance  9.96 ...
   cluster 16: chars 18..19 y     advance  7.47 ...
   cluster 17: chars 19..20 e     advance  8.48 ... safe_break false
```

Things to notice:

- **20 characters became 18 shaped clusters.** `ffi` (chars 10..13) shaped into one glyph, id 446, one cluster. A shaped cluster is HarfBuzz's unit: a group of characters and the glyphs they turned into, with no finer correspondence available (`ShapedCluster`, `parley_engine/src/shape/data.rs:180`).
- **One character can be two glyphs.** The letter `ب` (cluster 6) is glyph 15 for the body plus glyph 309 for the dot, placed with an offset. The cluster's advance, 5.46, is the sum of the glyph advances.
- **The newline has no glyph and zero advance** (`process_shaped_clusters`, `parley_engine/src/shape/shaped_text.rs:542`). It still gets a cluster, so that lines can end on it and cursors can land after it.
- **"glyphs (stored): 2".** Sixteen of the eighteen clusters have exactly one glyph with no offset, so the glyph id is stored *inline* in the cluster's `glyph_offset` field and nothing goes into the glyph array (`inline true`). Only `ب`'s two glyphs are stored. This is a memory optimisation to be aware of when you read `glyph_offset` (`parley_engine/src/shape/data.rs:219`).
- **`safe_break`** is HarfBuzz's "unsafe to break" flag inverted. Arabic letters join, so most of them are not safe to break before: breaking a line there would change the glyph shapes. Parley's line breaker does not reshape yet, so it only breaks where analysis found a wrap opportunity, and these flags are stored for a future reshaping step (`#876`).
- **Font metrics travel with the run** (ascent, descent, leading, scaled to the font size) so that line box sizing never has to open the font again.
- **The space after the Arabic word (cluster 8) has advance 5.44 here but 3.96 in Parley's real layout below.** The probe's selector gave segment 4 Noto Kufi because its script is Arabic; Parley's coverage-based selector gives it Roboto, the first family that covers a space. Same engine, different `FontSelector`, different glyph. Font selection is the caller's decision in `parley_engine`.

The engine also prints **atoms**: the smallest spans whose edges are both grapheme and shaped-cluster boundaries. Here every atom is one cluster, except that the `ffi` atom covers one cluster and three graphemes:

```text
atom: chars 10..13 clusters 10..11 ffi advance 13.66 graphemes [10..11=4.55, 11..12=4.55, 12..13=4.55]
```

The ligature's advance is split evenly over its three graphemes, 13.66 / 3 = 4.55 each. That is where a caret inside "ffi" goes (`Graphemes::next`, `parley_engine/src/shape/atom.rs:670`).

## Stage 5: style metrics

After shaping, `resolve_style_metrics` (`parley/src/layout/style_metrics.rs:106`) computes a **span box** for each style in the table: the ascent and descent of the style's first available font plus the half-leading implied by its `line-height`. The root style's span box is the **strut**, the invisible box CSS puts on every line so that an empty line still has a height. For our two styles, Roboto at 16px gives ascent 14.84 and descent 3.91, which quantize to 15 and 4 (height 19); Roboto at 24px gives 22.27 and 5.86, which quantize to 22 and 6 (height 28). Keep these four numbers for the line box below.

## Stage 6: line breaking

Now the repeatable phase. `break_all_lines(Some(80.0))` (`parley/src/layout/layout.rs:228`) runs the greedy breaker in `BreakLines::break_next` (`parley/src/layout/line_break.rs:1083`). It walks the shaped runs **atom by atom**, adds each atom's advance to the running `x`, remembers the last atom that had a wrap opportunity before it, and when an atom does not fit, rewinds to that remembered atom and ends the line there.

With width 80 the probe printed three lines:

```text
width 61.61 full_width 65.58 height 70.00 lines 3

line: text 0..14 = "Hi مرحبا " break Regular
  metrics: baseline 22 line_height 32 block 0..32 advance 65.58 hanging 3.96 offset 0 inline 0..80
line: text 14..22 = "office!\n" break Explicit
  metrics: baseline 47 line_height 19 block 32..51 advance 43.75 hanging 0.00
line: text 22..25 = "Bye" break None
  metrics: baseline 66 line_height 19 block 51..70 advance 25.91 hanging 0.00
```

Follow the first line. Atoms are appended: `H` 17.11, `i` 5.82, space 3.96, then the five Arabic letters (34.72 together), then the space (3.96), giving x = 65.58. The next atom is `o`. There is a wrap opportunity before it, so the breaker saves a checkpoint (`mark_line_break_opportunity`, `parley/src/layout/line_break.rs:814`). Then it tries to add `o`: 65.58 + 9.12 = 74.70, fits. `ffi`: 74.70 + 13.66 = 88.36, does not fit. The breaker rewinds to the checkpoint and ends the line before `o` with reason `Regular` (`parley/src/layout/line_break.rs:1281`).

The line's `advance` is 65.58, but 3.96 of that is the trailing space, which **hangs** past the end (`hanging_advance`). Hanging whitespace does not count towards the layout width: `width` is 61.61 while `full_width` is 65.58. It also does not take part in alignment or justification. The [algorithms chapter](algorithms.md#hanging-whitespace) has the rules.

The second line ends at the newline with reason `Explicit`. The third ends at the end of text with reason `None`. Compare the same text at width 120, where everything up to the newline fits:

```text
width 109.33 full_width 109.33 height 51.00 lines 2
line: text 0..22 = "Hi مرحبا office!\n" break Explicit  advance 109.33 hanging 0.00
line: text 22..25 = "Bye" break None
```

The content widths that `calculate_content_widths` reports (`parley/src/layout/data.rs:418`) are `min 43.75 max 109.33`: the widest unbreakable fragment is `office!` (43.75), and the widest line with only forced breaks is the whole first paragraph (109.33). These are CSS `min-content` and `max-content`.

### The line box

Why is the first line 32 pixels tall with the baseline at 22? `finish_line` (`parley/src/layout/line_break.rs:1488`) collects every box on the line and takes the maximum extent above and below the baseline:

| Box on line 1 | above baseline | below baseline | source |
|---|---|---|---|
| strut (root style, Roboto 16) | 15 | 4 | always present (`reset_line`, `line_break.rs:943`) |
| span box of style 1 (Roboto 24) | 22 | 6 | `add_style` when `Hi` is appended |
| run box of the Arabic run (Noto Kufi 16, fallback font) | 21 | 10 | `run_box_metrics` (`data.rs:56`) because `line-height` is `normal` |
| **line box** | **22** | **10** | max of each column |

22 + 10 = 32, baseline at 22. The fallback font's taller metrics push the bottom down by six pixels even though no style asked for Noto Kufi. That is CSS behaviour (CSS Inline 3 section 4.1): under `line-height: normal`, fallback fonts grow the line. Lines 2 and 3 only have Roboto 16 boxes: 15 + 4 = 19, baselines at 32 + 15 = 47 and 51 + 15 = 66.

## Stage 7: alignment and bidi reordering

`align(Alignment::Start, ...)` (`parley/src/layout/alignment.rs:69`) sets each line's `offset`. For start alignment in a left-to-right paragraph that is 0, so the numbers above are final.

Bidi reordering already happened in `commit_line` (`parley/src/layout/line_break.rs:1820`): the line's items are put in visual order by `reorder_line_items` (`line_break.rs:1983`), which reverses runs of items at odd levels. Our first line has the items `Hi`, ` `, `مرحبا`, ` ` at levels 0, 0, 1, 0, so only the Arabic run is affected, and a single run reversed is itself. Within that run, the clusters are visited from the logical end when iterating visually (`Run::visual_clusters`, `parley/src/layout/run.rs:205`):

```text
run 2: text 3..13 = "مرحبا" rtl true advance 34.72
  cluster 11..13 ا  advance 5.06   <- logically last, drawn first (leftmost)
  cluster 9..11  ب  advance 5.46
  cluster 7..9   ح  advance 9.28
  cluster 5..7   ر  advance 5.74
  cluster 3..5   م  advance 9.18   <- logically first, drawn last (rightmost)
```

## Stage 8: reading the result

`Line::items()` (`parley/src/layout/line.rs:139`) yields `PositionedLayoutItem`s: glyph runs split by style, with a final offset and baseline, and inline boxes. For line 1:

```text
glyph run: style 1 offset  0.00 baseline 22 advance 22.93 glyphs 44@(0.00,22) 77@(17.11,22)
glyph run: style 0 offset 22.93 baseline 22 advance  3.96 glyphs 4@(22.93,22)
glyph run: style 0 offset 26.89 baseline 22 advance 34.72 glyphs 9@(26.89,22) 309@(34.61,22.30) 15@(31.95,22) 26@(37.41,22) 30@(46.69,22) 84@(52.43,22)
glyph run: style 0 offset 61.61 baseline 22 advance  3.96 glyphs 4@(61.61,22)
```

A renderer loops over these, loads the run's font (`run.font()`), and draws each glyph id at `(x, y)`. The Arabic glyphs are already in left-to-right paint order, and the dot of `ب` (glyph 309) sits at 34.61 with a y offset of 0.30, both computed by HarfRust.

Notice the glyph run for `Hi` has style 1 and the rest style 0. A `GlyphRun` never crosses a style boundary, even inside one shaped run, because the renderer needs one brush per run (`GlyphRunIter::next`, `parley/src/layout/line.rs:452`).

## Stage 9: cursors and hit testing

Finally the editing layer. `Cursor::from_byte_index(&layout, byte, Affinity::Downstream).geometry(&layout, 1.0)` gives a caret rectangle:

```text
cursor at byte  0: x  0.00 y 0..32
cursor at byte  2: x 22.93 y 0..32
cursor at byte  3: x 61.61 y 0..32
cursor at byte 13: x 61.61 y 0..32
cursor at byte 16: x 13.68 y 32..51
cursor at byte 17: x 18.23 y 32..51
cursor at byte 20: x 39.63 y 32..51
```

Bytes 3 and 13 give the same x. Byte 3 is the logical start of the Arabic word, which is its *right* edge visually (61.61). Byte 13 is the start of the space that follows the word, whose left edge is also 61.61. Two different text positions, one pixel position: this is the classic bidi caret problem, and `Affinity` (`parley/src/layout/cluster.rs:472`) exists to tell them apart. Bytes 16 and 17 are inside `ffi` (after the first `f` and after the second): 13.68 and 18.23 are 4.55 apart, the ligature's advance divided by three.

Hit testing goes the other way with `Cluster::from_point` (`parley/src/layout/cluster.rs:72`):

```text
point (5, 10)   -> cluster "H"  side Left
point (60, 10)  -> cluster "م"  side Right
point (100, 10) -> cluster " "  side Right   (past the end: nearest cluster, the hanging space)
point (10, 40)  -> cluster "f"  side Left    (second line)
```

## What to take away

- The pipeline is a chain of index translations: bytes to chars (analysis), chars to shaped clusters and glyphs (shaping), clusters to atoms (the breaker's unit), atoms to lines and line items (breaking), and back out to bytes for cursors.
- Everything before line breaking is computed once per `Layout`. Everything after it is recomputed by `break_all_lines` and `align`.
- Numbers you see in the API are shaped advances plus read-time spacing; the shaped advances themselves never change.

The next chapter is the [architecture map](architecture.md), which puts every file you just saw in its place.
