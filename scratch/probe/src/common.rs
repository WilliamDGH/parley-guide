#![allow(dead_code)]
use std::borrow::Cow;
use std::sync::Arc;

use parley::fontique::{Blob, Collection, CollectionOptions, SourceCache};
use parley::{FontContext, FontFamilyName};

pub const FAMILIES: &[FontFamilyName<'_>] = &[
    FontFamilyName::Named(Cow::Borrowed("Roboto")),
    FontFamilyName::Named(Cow::Borrowed("Noto Kufi Arabic")),
];

pub fn font_context() -> FontContext {
    let mut collection = Collection::new(CollectionOptions {
        shared: false,
        system_fonts: false,
    });
    for dir in parley_dev::font_dirs() {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            let ok = path
                .extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| ["ttf", "otf", "ttc", "otc"].contains(&e));
            if ok {
                let data = std::fs::read(&path).unwrap();
                collection.register_fonts(Blob::new(Arc::new(data)), None);
            }
        }
    }
    FontContext {
        collection,
        source_cache: SourceCache::default(),
    }
}

/// Reads a font file from `parley_dev/assets/fonts`, given a path relative to that folder.
pub fn font_bytes(rel: &str) -> &'static [u8] {
    // `parley_dev::font_dirs()` yields `.../assets/fonts/<family>`; its parent is the fonts root.
    let root = parley_dev::font_dirs().next().unwrap().parent().unwrap().to_path_buf();
    Box::leak(std::fs::read(root.join(rel)).unwrap().into_boxed_slice())
}

pub fn script_name(s: icu_properties::props::Script) -> String {
    static NAMES: icu_properties::PropertyNamesShortBorrowed<'static, icu_properties::props::Script> =
        icu_properties::PropertyNamesShort::new();
    NAMES.get(s).unwrap_or("????").to_string()
}

pub fn show(c: char) -> String {
    match c {
        '\n' => "\\n".into(),
        ' ' => "space".into(),
        c => c.to_string(),
    }
}

pub fn show_str(s: &str) -> String {
    s.chars().map(show).collect()
}
