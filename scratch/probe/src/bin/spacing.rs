//! Hanging whitespace, justification and letter spacing with real numbers.
#[path = "../common.rs"]
mod common;

use parley::{
    Alignment, AlignmentOptions, FontFamily, Layout, StyleProperty, TextWrapMode,
    WhiteSpaceCollapse,
};

fn build(
    fcx: &mut parley::FontContext,
    lcx: &mut parley::LayoutContext<()>,
    text: &str,
    f: impl FnOnce(&mut parley::RangedBuilder<'_, ()>),
) -> Layout<()> {
    let mut b = lcx.ranged_builder(fcx, text, 1.0, true);
    b.push_default(StyleProperty::FontFamily(FontFamily::List(common::FAMILIES.into())));
    b.push_default(StyleProperty::FontSize(16.0));
    f(&mut b);
    b.build(text)
}

fn lines(layout: &Layout<()>, text: &str) {
    for line in layout.lines() {
        let m = line.metrics();
        println!(
            "  line {:?} break {:?} advance {:.2} hanging {:.2} offset {:.2} inline_max {:.2}",
            &text[line.text_range()], line.break_reason(), m.advance, m.hanging_advance, m.offset, m.inline_max_coord
        );
        for run in line.runs() {
            let advances: Vec<String> = run.visual_clusters().map(|c| format!("{}={:.2}", common::show_str(&text[c.text_range()]), c.advance())).collect();
            println!("    run advance {:.2}: {}", run.advance(), advances.join(" "));
        }
    }
}

fn main() {
    let mut fcx = common::font_context();
    let mut lcx = parley::LayoutContext::<()>::new();

    println!("== Reference advances (Roboto 16px) ==");
    for s in ["a", "b", " ", "ab", "a b", "cd"] {
        let mut l = build(&mut fcx, &mut lcx, s, |b| {
            b.push_default(StyleProperty::WhiteSpaceCollapse(WhiteSpaceCollapse::Preserve));
            b.push_default(StyleProperty::TextWrapMode(TextWrapMode::NoWrap));
        });
        l.break_all_lines(None);
        println!("  {s:?}: full_width {:.2}", l.full_width());
    }

    println!("\n== Hanging whitespace: \"ab   cd\" at width 30 ==");
    for collapse in [WhiteSpaceCollapse::Collapse, WhiteSpaceCollapse::Preserve, WhiteSpaceCollapse::BreakSpaces] {
        let text = "ab   cd";
        let mut l = build(&mut fcx, &mut lcx, text, |b| {
            b.push_default(StyleProperty::WhiteSpaceCollapse(collapse));
        });
        let w = l.calculate_content_widths();
        l.break_all_lines(Some(30.0));
        l.align(Alignment::Start, AlignmentOptions::default());
        println!(" {collapse:?}: content widths min {:.2} max {:.2}; layout width {:.2} full_width {:.2} lines {}", w.min, w.max, l.width(), l.full_width(), l.len());
        lines(&l, text);
    }

    println!("\n== Conditional hanging: \"ab   \\ncd\" (Preserve), widths 60, 24, 20 ==");
    for width in [60.0, 24.0, 20.0] {
        let text = "ab   \ncd";
        let mut l = build(&mut fcx, &mut lcx, text, |b| {
            b.push_default(StyleProperty::WhiteSpaceCollapse(WhiteSpaceCollapse::Preserve));
        });
        l.break_all_lines(Some(width));
        l.align(Alignment::Start, AlignmentOptions::default());
        println!(" width {width}: layout width {:.2} full_width {:.2}", l.width(), l.full_width());
        lines(&l, text);
    }

    println!("\n== Justify: \"aa bb cc dd ee\" at width 60 ==");
    let text = "aa bb cc dd ee";
    let mut l = build(&mut fcx, &mut lcx, text, |_| {});
    l.break_all_lines(Some(60.0));
    for align in [Alignment::Start, Alignment::Justify, Alignment::End] {
        l.align(align, AlignmentOptions::default());
        println!(" {align:?}:");
        lines(&l, text);
    }

    println!("\n== Letter spacing 2px and word spacing 5px: \"ab cd\" ==");
    let text = "ab cd";
    for (ls, ws) in [(0.0, 0.0), (2.0, 0.0), (0.0, 5.0), (2.0, 5.0)] {
        let mut l = build(&mut fcx, &mut lcx, text, |b| {
            b.push_default(StyleProperty::LetterSpacing(ls));
            b.push_default(StyleProperty::WordSpacing(ws));
        });
        l.break_all_lines(None);
        println!(" letter {ls} word {ws}: width {:.2}", l.width());
        lines(&l, text);
    }

    println!("\n== Ligature with letter spacing: \"office\" ==");
    for ls in [0.0, 1.0] {
        let text = "office";
        let mut l = build(&mut fcx, &mut lcx, text, |b| {
            b.push_default(StyleProperty::LetterSpacing(ls));
        });
        l.break_all_lines(None);
        println!(" letter {ls}: width {:.2}", l.width());
        for line in l.lines() {
            for run in line.runs() {
                for c in run.clusters() {
                    let glyphs: Vec<u32> = c.glyphs().map(|g| g.id).collect();
                    println!("    {:?} advance {:.2} lig_start {} lig_cont {} glyphs {:?}", &text[c.text_range()], c.advance(), c.is_ligature_start(), c.is_ligature_continuation(), glyphs);
                }
            }
        }
    }

    println!("\n== Vertical align: \"x\" + super \"2\" + \"y\" ==");
    let text = "x2y";
    let mut l = build(&mut fcx, &mut lcx, text, |b| {
        b.push(StyleProperty::VerticalAlign(parley::VerticalAlign::SUPER), 1..2);
        b.push(StyleProperty::FontSize(10.0), 1..2);
    });
    l.break_all_lines(None);
    l.align(Alignment::Start, AlignmentOptions::default());
    for line in l.lines() {
        let m = line.metrics();
        println!("  line baseline {} line_height {} block {}..{} content {}..{}", m.baseline, m.line_height, m.block_min_coord, m.block_max_coord, m.content_block_min_coord, m.content_block_max_coord);
        for item in line.items() {
            if let parley::PositionedLayoutItem::GlyphRun(gr) = item {
                println!("    glyph run style {} size {} offset {:.2} baseline {:.2}", gr.style_index(), gr.run().font_size(), gr.offset(), gr.baseline());
            }
        }
    }
    println!("  root span metrics: {:?}", l.root_span_metrics());

    println!("\n== Line height: default (MetricsRelative 1.0) vs FontSizeRelative(1.5) vs Absolute(40) ==");
    for lh in [parley::LineHeight::MetricsRelative(1.0), parley::LineHeight::FontSizeRelative(1.5), parley::LineHeight::Absolute(40.0)] {
        let text = "Hi";
        let mut l = build(&mut fcx, &mut lcx, text, |b| {
            b.push_default(StyleProperty::LineHeight(lh));
        });
        l.break_all_lines(None);
        let m = l.get(0).unwrap().metrics();
        println!("  {lh:?}: baseline {} line_height {} block {}..{} content {}..{}", m.baseline, m.line_height, m.block_min_coord, m.block_max_coord, m.content_block_min_coord, m.content_block_max_coord);
    }
}
