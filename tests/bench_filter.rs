//! Ignored micro-benchmark for the per-keystroke filter path.
//!
//! Run with:
//! ```text
//! cargo test --release --test bench_filter -- --ignored --nocapture
//! ```
//!
//! It measures `FuzzyFinder::set_query` (the work done on every keystroke) by
//! typing a query one character at a time, so each step is a cache miss.

use ff::fuzzy::FuzzyFinder;
use std::time::Instant;

fn items(count: usize) -> Vec<String> {
    (0..count)
        .map(|i| format!("src/module_{}/file_{i}.rs", i % 97))
        .collect()
}

async fn bench_corpus(count: usize) {
    let mut finder = FuzzyFinder::with_items_async(items(count), false).await;

    // Warm up (page in the allocations, exercise the cache).
    finder.set_query("module".to_string()).await;
    finder.set_query(String::new()).await;

    let typed = "module_97/file_1234";
    let mut query = String::new();
    let mut total = std::time::Duration::ZERO;
    let mut worst = std::time::Duration::ZERO;

    for ch in typed.chars() {
        query.push(ch);
        let start = Instant::now();
        finder.set_query(query.clone()).await;
        let elapsed = start.elapsed();
        total += elapsed;
        worst = worst.max(elapsed);
    }

    let keystrokes = typed.chars().count() as u32;
    println!(
        "items={count:>7}  keystrokes={keystrokes:>2}  \
         mean={:>8.0}us  worst={:>8.0}us  final_matches={}",
        total.as_micros() as f64 / f64::from(keystrokes),
        worst.as_micros(),
        finder.get_filtered_items().len()
    );
}

#[tokio::test]
#[ignore = "performance benchmark"]
async fn bench_keystroke_latency() {
    for count in [10_000, 100_000] {
        bench_corpus(count).await;
    }
}
