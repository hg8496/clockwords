pub mod lang;
pub mod resolve;
pub mod scanner;
pub mod types;

pub use scanner::TimeExpressionScanner;
pub use types::*;

/// Create a scanner with all four languages enabled (EN, DE, FR, ES).
pub fn default_scanner() -> TimeExpressionScanner {
    scanner_for_languages(&["en", "de", "fr", "es"])
}

/// Create a scanner for specific languages.
///
/// Supported language ids: `"en"`, `"de"`, `"fr"`, `"es"`.
/// Languages are tried in the order given; earlier languages take priority
/// when deduplicating overlapping matches.
pub fn scanner_for_languages(lang_ids: &[&str]) -> TimeExpressionScanner {
    let languages: Vec<Box<dyn lang::LanguageParser>> = lang_ids
        .iter()
        .filter_map(|id| match *id {
            "en" => Some(Box::new(lang::en::English::new()) as Box<dyn lang::LanguageParser>),
            "de" => Some(Box::new(lang::de::German::new()) as Box<dyn lang::LanguageParser>),
            "fr" => Some(Box::new(lang::fr::French::new()) as Box<dyn lang::LanguageParser>),
            "es" => Some(Box::new(lang::es::Spanish::new()) as Box<dyn lang::LanguageParser>),
            _ => None,
        })
        .collect();

    TimeExpressionScanner::new(languages, ParserConfig::default())
}

// Tests have been moved to the `tests/` directory.
