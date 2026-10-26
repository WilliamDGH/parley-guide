# Glossary

Every term the guide uses, with a plain meaning and, where it fits, an analogy for someone who knows CSS, TypeScript or Java. Entries point at the type or function that embodies the term.

**Advance**
: How far the pen moves after drawing a glyph, cluster or atom, along the line. In horizontal text this is the width that counts for layout, which is not the same as the ink's width. `Glyph::advance`, `ShapedCluster::advance`, `Atom::advance()`. CSS analogy: the width a `display: inline` element contributes to its line, ignoring overflow.

**Affinity**
: Which side a caret sticks to when one byte position has two visual positions, at a line end or a direction change. `Downstream` means the caret belongs with the following character, `Upstream` with the preceding one. `Affinity` (`parley/src/layout/cluster.rs:472`). Analogy: in a browser's `Selection` API, the same offset can be shown at the end of one line or the start of the next.

**Aligned subtree**
: CSS Inline 3's term for a box plus all descendants whose `vertical-align` is relative to their parent. Parley tracks the *independent* ones: the root span box and each `vertical-align: top | bottom` box. Within one subtree all baselines are fixed relative to each other; subtrees are positioned against each other when the line is complete. `style_metrics.rs:14`.

**Analysis**
: The font-independent pass over the text: scripts, bidi levels, grapheme, word and line boundaries, emoji properties. `Analyzer`, `Analysis`, `CharInfo` in `parley_engine`. Analogy: everything a browser knows about a text node before it has picked fonts.

**Atom**
: The smallest span of shaped text whose edges are both grapheme and shaped-cluster boundaries. The unit of line breaking and of letter spacing; cannot be split without reshaping. `Atom` (`parley_engine/src/shape/atom.rs:481`). Analogy: the smallest piece you can move to the next line without changing what the glyphs look like.

**Baseline**
: The line glyphs sit on. `LineMetrics::baseline` is its y coordinate within the layout. Glyph `y` offsets are relative to it.

**Bidi, bidi level**
: Short for bidirectional. The Unicode Bidirectional Algorithm (UAX 9) assigns every character an embedding level: even for left-to-right, odd for right-to-left. `BidiLevel` (parlance), `BidiResolver` (`parley_engine/src/bidi.rs`). Analogy: CSS `direction` and `unicode-bidi`, computed for you.

**Break opportunity, soft wrap opportunity**
: A position where a line may end if it needs to. Comes from UAX 14 plus CSS rules. Stored on the character after the position: `CharInfo::is_soft_wrap_opportunity`. Opposite: a **mandatory** (forced, hard) break at a newline. Analogy: where `word-wrap` is allowed to wrap.

**Break reason**
: Why a line ended: `Regular` (a soft wrap), `Emergency` (an `overflow-wrap` break inside a word), `Explicit` (a newline), `None` (end of text). `BreakReason` (`parley/src/layout/data.rs:72`).

**Brush**
: The type Parley uses for colors without knowing what a color is. Any `Clone + PartialEq + Default + Debug` type. `Brush` (`parley/src/style/brush.rs`). Analogy: a generic type parameter `Layout<B>` like `List<T>`; Parley stores your `B` and hands it back.

**Builder**
: The object you push text and styles into to get a `Layout`. `RangedBuilder` (styles by byte range), `TreeBuilder` (nested spans, like HTML, with whitespace collapsing), `StyleRunBuilder` (precomputed style table). Analogy: the builder pattern in Java; or React elements before rendering.

**Character**
: One Unicode scalar value, one Rust `char`, one entry of `ShapedText::characters`. Not a byte and not a grapheme. `Character` (`parley_engine/src/shape/data.rs:15`). Analogy: one code point from `Array.from(str)` in JavaScript, not one UTF-16 unit.

**Cluster (`parley::Cluster`)**
: One grapheme inside one run on one line, as seen by the editing API: it has a text range, an advance, a style, and knows its neighbours in logical and visual order. `Cluster` (`parley/src/layout/cluster.rs:24`). Not to be confused with a shaped cluster.

**Content box, content block**
: The part of a line covered by the fonts' ascent and descent and by inline boxes, ignoring leading. `LineMetrics::content_block_min_coord` and `_max_coord`. Can be larger than the line box when line height is small. Analogy: CSS Inline 3's content area.

**Content widths**
: `min` is the width if every soft wrap opportunity is taken; `max` is the width if none is. `ContentWidths`, `Layout::calculate_content_widths`. Analogy: CSS `min-content` and `max-content`.

**Coverage**
: The fraction of a grapheme's characters a font has glyphs for. Used to pick a font. `Coverage` (`parley_engine/src/shape/cluster.rs:177`).

**Fallback (font fallback)**
: Choosing another font when the requested one lacks a character, per script and language. `FallbackMap`, `Query::set_fallbacks` in fontique. Analogy: the browser silently using a different font for the emoji in your `font-family: Arial` paragraph.

**Font context**
: `FontContext`: the font database plus a cache of loaded font files. Make one, keep it.

**Glyph**
: A shape in a font, identified by an integer id, with an advance and an offset. Not a character: one character may become several glyphs and several characters one glyph. `Glyph` (`parley_engine/src/glyph.rs`).

**Glyph run**
: A sequence of positioned glyphs on one line with one font and one style, ready to paint. `GlyphRun` (`parley/src/layout/line.rs:283`). Analogy: one `fillText` call.

**Grapheme (cluster)**
: What a user perceives as one character: a base letter with its combining marks, an emoji with its modifiers, a flag made of two regional indicators. Defined by UAX 29. The unit of caret movement. `Grapheme` (`parley_engine/src/shape/atom.rs:808`). Analogy: one element of `new Intl.Segmenter().segment(str)`.

**Hanging whitespace**
: Trailing whitespace that is allowed to stick out past the line's end so it does not count for fitting, width or alignment. `LineMetrics::hanging_advance`. Analogy: in CSS, the space at the end of a wrapped line that does not push the line over the container width.

**Inline box**
: A rectangle laid out inline with the text, such as an image or a widget. In-flow boxes occupy a U+FFFC character and take space; out-of-flow boxes take no space. `InlineBox`, `InlineBoxKind`. Analogy: `<img>` inside a paragraph (in-flow) or a `position: absolute` child (out-of-flow). Note the name clash: CSS calls every inline element's box an "inline box"; Parley calls those **span boxes**.

**Item, itemization**
: Cutting text into pieces that can be shaped in one go: same direction, script and shaping options. `Item`, `Segment`, `Itemizer` (`parley_engine/src/itemize.rs`).

**Layout context**
: `LayoutContext`: reusable scratch memory for building layouts. Make one per thread.

**Leading**
: Extra space a font recommends between lines (`FontMetrics::leading`), and more generally the space added above and below the content to reach the line height (half-leading). Analogy: the difference between `line-height` and the font's natural height, split in two.

**Line box**
: The rectangle a line occupies, from `block_min_coord` to `block_max_coord`, tall enough for every span box and inline box on it. Lines stack by line box. Analogy: CSS 2.2 section 9.4.2 line box.

**Line item**
: One run or out-of-flow box as it appears on one line, in visual order. `LineItemData`.

**Logical order, visual order**
: Logical is the order of the text in memory (reading order). Visual is left to right on screen. They differ inside and around right-to-left text. `Run::clusters()` is logical, `Run::visual_clusters()` is visual.

**Quantize**
: Round vertical metrics to whole pixels so that text is not blurry. Does not affect horizontal advances. The `quantize` argument of the builders.

**Resolved style**
: A style with its lengths scaled and its font family names turned into ids. `ResolvedStyle` (`parley/src/resolve/mod.rs:407`). Analogy: a computed style versus a specified style in CSS.

**Run (shaped run)**
: A maximal sequence of consecutive characters shaped with one font, one direction, one script and one set of shaping options. `ShapedRun` in the engine, `RunData` plus the `Run` view in `parley`. Analogy: one HarfBuzz call.

**Scale**
: The display scale factor, like `devicePixelRatio`. Multiplies every length at resolution time.

**Script**
: The writing system of a character: Latin, Arabic, Han, and so on. `Common` and `Inherited` characters take the script of their neighbours. Determines shaping rules and fallback fonts. `Script` (parlance), ISO 15924 codes such as `Latn`, `Arab`.

**Segment**
: A piece of an item with one bidi level and one script. `Segment` (`parley_engine/src/itemize.rs:25`).

**Shaped cluster**
: HarfBuzz's cluster: a group of characters and the glyphs they produced, with no finer mapping available. A ligature is one shaped cluster over several graphemes; a base plus mark may be one or two. `ShapedCluster` (`parley_engine/src/shape/data.rs:180`).

**Shaped text**
: The complete output of shaping for a paragraph: runs, characters, clusters, glyphs, fonts, in parallel vectors. `ShapedText`. Everything after it is layout.

**Shaping**
: Turning characters into positioned glyphs using the font's rules: ligatures, kerning, Arabic joining forms, Indic reordering, mark placement. Done by HarfRust. Analogy: the step a browser performs between "I have this string in this font" and "I have these glyphs at these positions"; there is no CSS property for it because you never see it.

**Span, span box**
: A styled range of text (an entry of the style table) and the box it generates on each line it touches: ascent, descent and half-leading from its first available font and line height. The root span box is the **strut**. `StyleMetrics`. Analogy: the box of a `<span>`; CSS calls it an inline box.

**Strut**
: The invisible box of the root style that is present on every line so that even an empty line has the paragraph's line height. CSS 2.2 section 10.8.1 uses the same word.

**Style index**
: A `u16` into the style table. Index 0 is the root. Carried by every character and cluster.

**Style run**
: A byte range with one style index, produced by the builders and consumed by shaping. `StyleRun` (`parley/src/resolve/mod.rs:36`).

**Synthesis**
: Faking bold (emboldening) or italic (skewing) when the family has no such face, or driving a variable font's axes to get them. `Synthesis` (fontique). Analogy: CSS `font-synthesis`.

**UAX 9, UAX 14, UAX 29, UTS 51**
: The Unicode standards for bidirectional text, line breaking, text segmentation (graphemes and words), and emoji. Parley gets 14 and 29 from ICU4X, implements 9 itself, and uses a translation of `c-emoji` for 51.

**U+FFFC**
: The OBJECT REPLACEMENT CHARACTER, a placeholder in the text for an in-flow inline box.

**Word separator**
: Whitespace that gets `word-spacing` and justification: space and no-break space in Parley (`is_word_separator`, `parley/src/layout/spacing.rs:18`).
