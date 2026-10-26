//! Runs one string through analysis, shaping and layout and prints the numbers.
#[path = "../common.rs"]
mod common;

use std::sync::Arc;

use parley::fontique::{Blob, Synthesis};
use parley::{
    Affinity, Alignment, AlignmentOptions, Cursor, FontData, FontFamily, PositionedLayoutItem,
    StyleProperty,
};
use parley_engine::itemize::{Item, Segment};
use parley_engine::shape::CharCluster;
use parley_engine::{
    Analysis, AnalysisDataSources, AnalysisOptions, Analyzer, FontInstance, FontInstanceRef,
    FontSelector, ShapeOptions, ShapedText, Shaper,
};
use parlance::Script;

const TEXT: &str = "Hi مرحبا office!\nBye";
const BIG: std::ops::Range<usize> = 0..2; // "Hi" at 24px
fn max_width() -> f32 { std::env::args().nth(1).and_then(|a| a.parse().ok()).unwrap_or(120.0) }

struct TwoFonts {
    latin: FontInstance,
    arabic: FontInstance,
}

impl FontSelector for TwoFonts {
    fn select_font(
        &mut self,
        segment: &Segment,
        _options: &ShapeOptions<'_>,
        _cluster: &mut CharCluster,
    ) -> Option<FontInstanceRef<'_>> {
        if segment.script == Script::from_bytes(*b"Arab") {
            Some(self.arabic.as_ref())
        } else {
            Some(self.latin.as_ref())
        }
    }
}

fn instance(bytes: &'static [u8]) -> FontInstance {
    FontInstance {
        font: FontData::new(Blob::new(Arc::new(bytes)), 0),
        synthesis: Synthesis::default(),
    }
}

fn main() {
    println!("TEXT = {TEXT:?}  ({} bytes, {} chars)\n", TEXT.len(), TEXT.chars().count());

    // ---------------------------------------------------------------- analysis
    let mut analyzer = Analyzer::new();
    let mut analysis = Analysis::new();
    analyzer.analyze(
        TEXT,
        &AnalysisOptions::default(),
        &mut analysis,
        &AnalysisDataSources::default(),
    );
    println!("== Analysis ==");
    println!("paragraph_level = {:?}", analysis.paragraph_level());
    println!("{:>4} {:>4} {:<6} {:<5} {:>5} {:>8} {:>5} {:>4}", "char", "byte", "text", "scr", "bidi", "grapheme", "wrap", "word");
    let levels = analysis.bidi_levels();
    for (i, ((byte, ch), info)) in TEXT.char_indices().zip(analysis.char_info()).enumerate() {
        let level = levels.get(i).map(|l| l.to_u8()).unwrap_or(analysis.paragraph_level().to_u8());
        println!(
            "{:>4} {:>4} {:<6} {:<5} {:>5} {:>8} {:>5} {:>4}",
            i,
            byte,
            common::show(ch),
            common::script_name(info.script),
            level,
            info.is_grapheme_start(),
            info.is_soft_wrap_opportunity(),
            info.is_word_boundary(),
        );
    }

    // ---------------------------------------------------------------- shaping
    let char_count = TEXT.chars().count() as u32;
    let big_chars = TEXT[BIG].chars().count() as u32;
    let mut style_indices = vec![0u16; char_count as usize];
    for s in &mut style_indices[..big_chars as usize] {
        *s = 1;
    }
    let items = [
        Item { char_end: big_chars, options: ShapeOptions { font_size: 24.0, language: None, features: &[], variations: &[] } },
        Item { char_end: char_count, options: ShapeOptions { font_size: 16.0, language: None, features: &[], variations: &[] } },
    ];
    let selector = TwoFonts {
        latin: instance(common::font_bytes("roboto_fonts/Roboto-Regular.ttf")),
        arabic: instance(common::font_bytes("noto_fonts/NotoKufiArabic-Regular.otf")),
    };
    let mut shaper = Shaper::default();
    let mut shaped = ShapedText::new();
    shaper.shape_text(TEXT, &analysis, &style_indices, items, selector, false, &mut shaped);

    println!("\n== ShapedText ==");
    println!("fonts: {}", shaped.fonts().len());
    println!("characters: {}  shaped_clusters: {}  glyphs (stored): {}", shaped.characters().len(), shaped.shaped_clusters().len(), shaped.glyphs().len());
    for (i, run) in shaped.runs().iter().enumerate() {
        println!(
            "run {i}: bytes {:?} chars {:?} clusters {:?} glyphs {:?} font#{} size {} bidi {} script {} ascent {:.2} descent {:.2} leading {:.2}",
            run.range.byte_range, run.range.char_range, run.shaped_clusters_range, run.glyphs_range,
            run.font_index, run.font_size, run.bidi_level.to_u8(), run.script,
            run.font_metrics.ascent, run.font_metrics.descent, run.font_metrics.leading
        );
        let slice = shaped.run_slice(i as u32);
        for ci in slice.shaped_clusters_range() {
            let c = &shaped.shaped_clusters()[ci as usize];
            let text: String = TEXT.chars().skip(c.chars_range().start as usize).take(c.char_len() as usize).map(common::show).collect();
            let glyphs: Vec<String> = slice.shaped_cluster_glyphs(ci).map(|g| format!("{}@{:.2}", g.id, g.advance)).collect();
            println!(
                "   cluster {ci}: chars {:?} {:<8} advance {:>6.2} glyph_len {} inline {} grapheme_start {} safe_break {} ws {:?} style {}  glyphs [{}]",
                c.chars_range(), text, c.advance, c.glyph_len(), c.has_inline_glyph(), c.is_grapheme_start(), c.is_safe_to_break_before(), c.whitespace(), c.style_index, glyphs.join(", ")
            );
        }
        for atom in slice.atoms_start() {
            let text: String = TEXT.chars().skip(atom.char_range().start as usize).take(atom.char_range().len()).map(common::show).collect();
            let graphemes: Vec<String> = atom.graphemes_start().map(|g| format!("{:?}={:.2}", g.char_range(), g.advance())).collect();
            println!(
                "   atom: chars {:?} clusters {:?} {:<8} advance {:>6.2} wrap_before {} safe_break {} graphemes [{}]",
                atom.char_range(), atom.shaped_clusters_range(), text, atom.advance(), atom.is_soft_wrap_opportunity_before(), atom.is_safe_to_break_before(), graphemes.join(", ")
            );
        }
    }

    // ---------------------------------------------------------------- layout
    let mut fcx = common::font_context();
    let mut lcx = parley::LayoutContext::<()>::new();
    let mut builder = lcx.ranged_builder(&mut fcx, TEXT, 1.0, true);
    builder.push_default(StyleProperty::FontFamily(FontFamily::List(common::FAMILIES.into())));
    builder.push_default(StyleProperty::FontSize(16.0));
    builder.push(StyleProperty::FontSize(24.0), BIG);
    let mut layout = builder.build(TEXT);
    let widths = layout.calculate_content_widths();
    let max_width = max_width();
    layout.break_all_lines(Some(max_width));
    layout.align(Alignment::Start, AlignmentOptions::default());

    println!("\n== Layout (max_advance = {max_width}) ==");
    println!("styles in table: {}", layout.styles().len());
    println!("content widths: min {:.2} max {:.2}", widths.min, widths.max);
    println!("width {:.2} full_width {:.2} height {:.2} lines {}", layout.width(), layout.full_width(), layout.height(), layout.len());
    for line in layout.lines() {
        let m = line.metrics();
        println!(
            "\nline: text {:?} = {:?} break {:?}\n  metrics: baseline {} line_height {} block {}..{} content_block {}..{} advance {:.2} hanging {:.2} offset {} inline {}..{}",
            line.text_range(), &TEXT[line.text_range()], line.break_reason(),
            m.baseline, m.line_height, m.block_min_coord, m.block_max_coord, m.content_block_min_coord, m.content_block_max_coord,
            m.advance, m.hanging_advance, m.offset, m.inline_min_coord, m.inline_max_coord
        );
        println!("  runs in visual order:");
        for run in line.runs() {
            println!(
                "    run {}: text {:?} = {:?} rtl {} size {} advance {:.2} clusters {}",
                run.index(), run.text_range(), &TEXT[run.text_range()], run.is_rtl(), run.font_size(), run.advance(), run.len()
            );
            for cluster in run.visual_clusters() {
                let glyphs: Vec<String> = cluster.glyphs().map(|g| g.id.to_string()).collect();
                println!(
                    "      cluster {:?} {:<8} advance {:>6.2} lig_start {} lig_cont {} word {} style {} glyphs [{}]",
                    cluster.text_range(), common::show_str(&TEXT[cluster.text_range()]), cluster.advance(), cluster.is_ligature_start(), cluster.is_ligature_continuation(), cluster.is_word_boundary(), cluster.style_index(), glyphs.join(",")
                );
            }
        }
        println!("  positioned items:");
        for item in line.items() {
            match item {
                PositionedLayoutItem::GlyphRun(gr) => {
                    let glyphs: Vec<String> = gr.positioned_glyphs().map(|g| format!("{}@({:.2},{:.2})", g.id, g.x, g.y)).collect();
                    println!("    glyph run: style {} offset {:.2} baseline {:.2} advance {:.2} glyphs {}", gr.style_index(), gr.offset(), gr.baseline(), gr.advance(), glyphs.join(" "));
                }
                PositionedLayoutItem::InlineBox(b) => println!("    inline box {:?}", b),
            }
        }
    }

    println!("\n== Cursor / hit test ==");
    for byte in [0, 2, 3, 13, 16, 17, 20] {
        let cursor = Cursor::from_byte_index(&layout, byte, Affinity::Downstream);
        let g = cursor.geometry(&layout, 1.0);
        println!("cursor at byte {byte:>2} (downstream): x {:.2} y {:.2}..{:.2}", g.x0, g.y0, g.y1);
    }
    for (x, y) in [(5.0, 10.0), (60.0, 10.0), (100.0, 10.0), (10.0, 40.0)] {
        match parley::Cluster::from_point(&layout, x, y) {
            Some((c, side)) => println!("point ({x}, {y}) -> cluster {:?} = {:?} side {:?}", c.text_range(), &TEXT[c.text_range()], side),
            None => println!("point ({x}, {y}) -> none"),
        }
    }
}
