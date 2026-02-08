use chrono::Utc;
use clockwords::default_scanner;
use std::time::Instant;

fn main() {
    let scanner = default_scanner();
    let now = Utc::now();

    // Test cases matching the README scenarios
    let scenarios = vec![
        (
            "No keywords (fast rejection)",
            "This text has absolutely no time related words in it. It should be rejected very quickly.",
            1_000_000, // higher iterations for very fast operations
        ),
        (
            "Short sentence (1 match)",
            "I will see you tomorrow at 5pm.",
            100_000,
        ),
        (
            "Paragraph (multiple matches)",
            "I saw him yesterday. He said he would come back in 2 days. Maybe last week was better.",
            100_000,
        ),
    ];

    println!(
        "{:<35} | {:<15} | {:<15}",
        "Scenario", "Avg Time (ns)", "Avg Time (µs)"
    );
    println!("{:-<35}-|-{:-<15}-|-{:-<15}", "", "", "");

    for (name, text, iterations) in scenarios {
        // Warmup
        for _ in 0..1000 {
            scanner.scan(text, now);
        }

        let start = Instant::now();
        for _ in 0..iterations {
            scanner.scan(text, now);
        }
        let duration = start.elapsed();
        let avg_ns = duration.as_nanos() as f64 / iterations as f64;
        let avg_us = avg_ns / 1000.0;

        println!("{:<35} | {:<15.2} | {:<15.4}", name, avg_ns, avg_us);
    }
}
