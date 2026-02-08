# clockwords

**Find and resolve natural-language time expressions in text.**

[![Crates.io](https://img.shields.io/crates/v/clockwords.svg)](https://crates.io/crates/clockwords)
[![Docs.rs](https://docs.rs/clockwords/badge.svg)](https://docs.rs/clockwords)
[![License](https://img.shields.io/crates/l/clockwords.svg)](LICENSE)

`clockwords` scans free-form text for relative time expressions like *"the last hour"*, *"yesterday at 3pm"*, or *"vor 3 Tagen"* and returns their byte-offset spans together with resolved `DateTime<Utc>` values. It supports **English**, **German**, **French**, and **Spanish** out of the box.

Built for **real-time GUI applications** (time-tracking, note-taking, calendars) where the user types naturally and the app highlights detected time references as they appear.

## Features

- **Four languages**: English, German, French, Spanish
- **Byte-offset spans**: Directly usable for text highlighting in any GUI framework
- **Resolved times**: Every match resolves to a concrete `DateTime<Utc>` point or range
- **Incremental typing support**: Detects partial matches (e.g. `"yester"` while the user is still typing `"yesterday"`)
- **Accent-tolerant**: Handles `días`/`dias`, `à`/`a`, `mañana`/`manana`, `dernière`/`derniere`
- **Fast rejection**: Aho-Corasick keyword prefilter skips text with no time-related words in sub-microsecond time
- **Zero allocations on rejection**: If no keywords are found, `scan()` returns immediately
- **No unsafe code**
- **Defensive**: All internal date arithmetic returns `Option` — no panics from edge-case dates

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
clockwords = "0.1"
```

### Basic Usage

```rust
use clockwords::{default_scanner, ResolvedTime};
use chrono::Utc;

fn main() {
    // Create a scanner with all four languages enabled
    let scanner = default_scanner();
    let now = Utc::now();

    let text = "The last hour I coded the initial code for the time library";
    let matches = scanner.scan(text, now);

    for m in &matches {
        println!(
            "Found '{}' at bytes {}..{} ({:?})",
            &text[m.span.as_range()],
            m.span.start,
            m.span.end,
            m.kind,
        );

        match &m.resolved {
            ResolvedTime::Point(dt) => println!("  Resolved to: {dt}"),
            ResolvedTime::Range { start, end } => {
                println!("  Resolved to: {start} .. {end}")
            }
        }
    }
}
```

**Output:**
```
Found 'The last hour' at bytes 0..13 (TimeRange)
  Resolved to: 2026-02-08T12:30:00Z .. 2026-02-08T13:30:00Z
```

### Select Specific Languages

```rust
use clockwords::scanner_for_languages;

// Only English and German
let scanner = scanner_for_languages(&["en", "de"]);
```

## Supported Expressions

### Relative Days

| Language | Examples |
|----------|----------|
| English  | `today`, `tomorrow`, `yesterday` |
| German   | `heute`, `morgen`, `gestern` |
| French   | `aujourd'hui`, `demain`, `hier` |
| Spanish  | `hoy`, `mañana`, `ayer` |

Resolves to a full-day `Range` (midnight to midnight).

### Day Offsets

| Language | Examples |
|----------|----------|
| English  | `in 4 days`, `two days ago`, `in three days` |
| German   | `in 3 Tagen`, `vor zwei Tagen` |
| French   | `dans 3 jours`, `il y a deux jours` |
| Spanish  | `en 3 días`, `hace 2 dias` |

Supports both digits and written-out number words (1–30).

### Time Specifications

| Language | Examples |
|----------|----------|
| English  | `at 3pm`, `at 3 am`, `13 o'clock` |
| German   | `um 15 Uhr` |
| French   | `à 13h` |
| Spanish  | `a las 3` |

Resolves to a `Point` in time.

### Time Ranges

| Language | Examples |
|----------|----------|
| English  | `the last hour`, `last minute`, `between 9 and 12` |
| German   | `die letzte Stunde`, `von 9 bis 12 Uhr`, `zwischen 9 und 12` |
| French   | `la dernière heure`, `entre 9 et 12 heures` |
| Spanish  | `la última hora`, `entre las 9 y las 12` |

### Combined Expressions

Day + time specification or day + time range in a single expression:

| Language | Examples |
|----------|----------|
| English  | `yesterday at 3pm`, `tomorrow between 9 and 12` |
| German   | `gestern um 15 Uhr`, `gestern von 9 bis 12 Uhr` |
| French   | `hier à 13h`, `hier entre 9 et 12 heures` |
| Spanish  | `ayer a las 3`, `ayer entre las 9 y las 12` |

## Architecture

### How Scanning Works

```
Input text
    │
    ▼
┌─────────────────────┐
│ Aho-Corasick        │  Fast keyword check (~ns)
│ Prefilter           │  Rejects text with no time words
└─────────┬───────────┘
          │ keywords found
          ▼
┌─────────────────────┐
│ Per-Language         │  Regex rules with resolver closures
│ Grammar Rules       │  Run for each enabled language
└─────────┬───────────┘
          │ raw matches
          ▼
┌─────────────────────┐
│ Deduplication       │  Prefer Complete > Partial, longer > shorter
│ & Sorting           │  Remove overlapping inferior matches
└─────────┬───────────┘
          │
          ▼
     Vec<TimeMatch>
```

### Buffer-Rescan Strategy

Rather than maintaining an incremental parser state machine, `clockwords` re-scans the full text buffer on every call to `scan()`. This is the right trade-off for GUI text input:

- Input buffers are typically < 1 KB
- Full regex scan of a short buffer completes in microseconds
- Dramatically simpler than maintaining parser state across edits
- No edge cases around cursor position, insertions, or deletions

### Type Overview

| Type | Description |
|------|-------------|
| `TimeExpressionScanner` | Main entry point — holds language parsers and prefilter |
| `TimeMatch` | A single match result: span + confidence + resolved time + kind |
| `Span` | Byte-offset range (`start..end`) for slicing the original text |
| `ResolvedTime` | `Point(DateTime<Utc>)` or `Range { start, end }` |
| `MatchConfidence` | `Partial` (user still typing) or `Complete` |
| `ExpressionKind` | `RelativeDay`, `RelativeDayOffset`, `TimeSpecification`, `TimeRange`, `Combined` |
| `ParserConfig` | Settings: `report_partial` (default `true`), `max_matches` (default `10`) |

## GUI Integration

`clockwords` is designed for real-time text highlighting. Here's how to wire it up:

```rust
use clockwords::{default_scanner, MatchConfidence, TimeExpressionScanner};
use chrono::Utc;

struct App {
    scanner: TimeExpressionScanner,
}

impl App {
    fn new() -> Self {
        Self {
            scanner: default_scanner(),
        }
    }

    /// Call this on every keystroke
    fn on_text_changed(&self, text: &str) {
        let matches = self.scanner.scan(text, Utc::now());

        for m in &matches {
            let range = m.span.start..m.span.end;
            let style = match m.confidence {
                MatchConfidence::Complete => "solid_underline",
                MatchConfidence::Partial  => "dotted_underline",
            };
            // Apply `style` to the character range in your text widget
            println!("Highlight bytes {range:?} with {style}");
        }
    }
}
```

### Partial Match Highlighting

When the user types `"I worked yester"`, the scanner returns a **Partial** match on `"yester"`. Your GUI can show a dimmed or dotted underline to hint that a time expression is being formed. Once the user completes `"yesterday"`, the match upgrades to **Complete** with a fully resolved time.

To disable partial matching:

```rust
use clockwords::{ParserConfig, TimeExpressionScanner};

let config = ParserConfig {
    report_partial: false,
    ..Default::default()
};
```

## Adding a New Language

1. Create `src/lang/xx.rs` (copy an existing language file as a template)
2. Implement the `LanguageParser` trait:
   - `lang_id()` — return the ISO 639-1 code (e.g. `"it"`)
   - `keywords()` — return Aho-Corasick trigger words
   - `keyword_prefixes()` — return typing prefixes (length >= 3)
   - `parse()` — call `apply_rules()` with your `GrammarRule` list
3. Add number-word mappings to `src/lang/numbers.rs`
4. Register the language in `src/lib.rs` → `scanner_for_languages()`
5. Add tests in `src/lib.rs`

Each `GrammarRule` is a compiled regex paired with a resolver closure:

```rust
GrammarRule {
    pattern: Regex::new(r"(?i)\b(?P<day>oggi|domani|ieri)\b").unwrap(),
    kind: ExpressionKind::RelativeDay,
    resolver: |caps, now| {
        let offset = match caps.name("day")?.as_str().to_lowercase().as_str() {
            "oggi" => 0,
            "domani" => 1,
            "ieri" => -1,
            _ => return None,
        };
        resolve::resolve_relative_day(offset, now)
    },
}
```

## Performance

| Scenario | Approximate Time |
|----------|-----------------|
| No keywords in text (fast rejection) | < 10 µs |
| Short sentence with 1 match | < 20 µs |
| Paragraph with multiple matches | < 25 µs |
| Full rescan on keystroke (typical) | < 50 µs |

The Aho-Corasick prefilter means that text without any time-related words is rejected in microseconds — the regex engine is never invoked.

## Running Tests

```bash
cargo test
```

The test suite covers:
- All four languages with various expression types
- Accent-tolerant variants (with and without diacritics)
- Embedded expressions in longer sentences
- Incremental/partial matching
- Edge cases (empty input, no false positives)
- Cross-language default scanner

## Running the TUI Demo

An interactive terminal demo is included:

```bash
cargo run --example tui_demo
```

Type time expressions and watch them get parsed in real time. Press **ESC** to quit.

## Dependencies

| Crate | Purpose |
|-------|---------|
| [`chrono`](https://crates.io/crates/chrono) | Date/time types and arithmetic |
| [`regex`](https://crates.io/crates/regex) | Per-language grammar patterns |
| [`aho-corasick`](https://crates.io/crates/aho-corasick) | Fast multi-keyword prefilter |

## License

Licensed under the Apache License, Version 2.0 ([LICENSE](LICENSE) or <http://www.apache.org/licenses/LICENSE-2.0>).
