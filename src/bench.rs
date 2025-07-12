use crate::fuzzy::FuzzyFinder;
use std::time::Instant;

/// Benchmark filtering performance with the given items and query.
pub fn benchmark_filtering(
    items: Vec<String>,
    query: &str,
    iterations: usize,
) -> (f64, Vec<String>) {
    let mut fuzzy_finder = FuzzyFinder::new(items, false);
    fuzzy_finder.query = query.to_string();

    let start = Instant::now();
    for _ in 0..iterations {
        fuzzy_finder.update_filter();
    }
    let duration = start.elapsed();

    let avg_time = duration.as_micros() as f64 / iterations as f64;
    (avg_time, fuzzy_finder.filtered_items)
}

/// Calculate average time per iteration from benchmark results.
pub fn calculate_benchmark_stats(duration: std::time::Duration, iterations: usize) -> f64 {
    duration.as_micros() as f64 / iterations as f64
}

/// Format benchmark results as a string.
pub fn format_benchmark_result(query: &str, avg_time: f64, result_count: usize) -> String {
    format!(
        "  Query '{}': {:.2}μs avg, {} results",
        query, avg_time, result_count
    )
}

/// Get benchmark dataset sizes.
pub fn get_benchmark_sizes() -> Vec<usize> {
    vec![100, 1000, 10000, 50000]
}

/// Get benchmark queries for testing.
pub fn get_benchmark_queries() -> Vec<&'static str> {
    vec!["item", "item_1", "item_12", "item_123"]
}

/// Get file path benchmark queries.
pub fn get_file_path_queries() -> Vec<&'static str> {
    vec![
        "src",         // Common prefix
        "main",        // Common name
        "rs",          // Common extension
        "test",        // Common substring
        "xyz",         // Rare substring
        "src/main",    // Path-like
        "main.rs",     // File-like
        "src/main.rs", // Full path
        "SRC",         // Case insensitive
        "sRc",         // Mixed case
    ]
}

/// Get realistic dataset queries.
pub fn get_realistic_queries() -> Vec<&'static str> {
    vec!["src", "main", "rs", "test", "doc", "target"]
}

/// Get large dataset queries.
pub fn get_large_dataset_queries() -> Vec<&'static str> {
    vec!["item", "item_1", "item_12", "item_123", "item_1234"]
}

/// Get sizes for parallel vs sequential benchmarks.
pub fn get_parallel_vs_sequential_sizes() -> Vec<usize> {
    vec![1000, 5000, 10000, 50000]
}

/// Get queries for parallel vs sequential benchmarks.
pub fn get_parallel_vs_sequential_queries() -> Vec<&'static str> {
    vec!["item", "item_1", "item_12"]
}

/// Generate sequential items for benchmarking.
pub fn generate_sequential_items(count: usize) -> Vec<String> {
    (0..count).map(|i| format!("item_{:05}", i)).collect()
}

/// Generate file paths for benchmarking.
pub fn generate_file_paths(count: usize) -> Vec<String> {
    let extensions = ["rs", "toml", "md", "txt", "json", "yaml", "yml"];
    let directories = ["src", "tests", "docs", "examples", "target", "benches"];
    let names = [
        "main", "lib", "config", "utils", "helpers", "types", "traits",
    ];

    (0..count)
        .map(|i| {
            let dir = directories[i % directories.len()];
            let name = names[i % names.len()];
            let ext = extensions[i % extensions.len()];
            format!("{}/{}.{}", dir, name, ext)
        })
        .collect()
}

/// Generate a realistic dataset for benchmarking.
pub fn generate_realistic_dataset() -> Vec<String> {
    vec![
        "src/main.rs".to_string(),
        "src/config.rs".to_string(),
        "src/input.rs".to_string(),
        "src/fuzzy.rs".to_string(),
        "src/tui.rs".to_string(),
        "src/bench.rs".to_string(),
        "Cargo.toml".to_string(),
        "Cargo.lock".to_string(),
        "README.md".to_string(),
        "target/debug/fuzzy-finder".to_string(),
        "target/release/fuzzy-finder".to_string(),
        "src/components/button.rs".to_string(),
        "src/components/input.rs".to_string(),
        "src/utils/helpers.rs".to_string(),
        "tests/integration_test.rs".to_string(),
        "docs/api.md".to_string(),
        "docs/usage.md".to_string(),
    ]
}

/// Run a single benchmark with given parameters.
pub fn run_single_benchmark(
    items: Vec<String>,
    query: &str,
    iterations: usize,
) -> (f64, Vec<String>) {
    let mut fuzzy_finder = FuzzyFinder::new(items, false);
    fuzzy_finder.query = query.to_string();

    let start = Instant::now();
    for _ in 0..iterations {
        fuzzy_finder.update_filter();
    }
    let duration = start.elapsed();

    let avg_time = calculate_benchmark_stats(duration, iterations);
    (avg_time, fuzzy_finder.filtered_items)
}

/// Benchmark parallel vs sequential filtering performance.
pub fn benchmark_parallel_vs_sequential() {
    println!("\n=== Parallel vs Sequential Benchmark ===");

    let sizes = get_parallel_vs_sequential_sizes();
    let queries = get_parallel_vs_sequential_queries();

    for size in sizes {
        println!("\nDataset size: {} items", size);
        let items = generate_sequential_items(size);

        for query in &queries {
            let (avg_time, results) = benchmark_filtering(items.clone(), query, 50);
            println!(
                "  Query '{}': {:.2}μs avg, {} results",
                query,
                avg_time,
                results.len()
            );
        }
    }
}

/// Benchmark performance with different dataset sizes.
pub fn benchmark_dataset_sizes() {
    println!("=== Dataset Size Benchmarks ===");

    let sizes = get_benchmark_sizes();
    let queries = get_benchmark_queries();

    for size in sizes {
        println!("\nDataset size: {} items", size);
        let items = generate_sequential_items(size);

        for query in &queries {
            let (avg_time, results) = benchmark_filtering(items.clone(), query, 100);
            println!(
                "  Query '{}': {:.2}μs avg, {} results",
                query,
                avg_time,
                results.len()
            );
        }
    }
}

/// Benchmark performance with different query types.
pub fn benchmark_query_types() {
    println!("\n=== Query Type Benchmarks ===");

    let items = generate_file_paths(10000);
    let queries = get_file_path_queries();

    println!("Dataset: {} file paths", items.len());

    for query in queries {
        let (avg_time, results) = benchmark_filtering(items.clone(), query, 100);
        println!(
            "  Query '{}': {:.2}μs avg, {} results",
            query,
            avg_time,
            results.len()
        );
    }
}

pub fn benchmark_realistic_dataset() {
    println!("\n=== Realistic Dataset Benchmark ===");

    let items = generate_realistic_dataset();
    let queries = get_realistic_queries();

    println!("Dataset: {} realistic file paths", items.len());

    for query in queries {
        let (avg_time, results) = benchmark_filtering(items.clone(), query, 1000);
        println!(
            "  Query '{}': {:.2}μs avg, {} results",
            query,
            avg_time,
            results.len()
        );
    }
}

pub fn benchmark_large_dataset() {
    println!("\n=== Large Dataset Benchmark ===");

    let items = generate_sequential_items(10000);
    let queries = get_large_dataset_queries();

    println!("Dataset: {} sequential items", items.len());

    for query in queries {
        let (avg_time, results) = benchmark_filtering(items.clone(), query, 100);
        println!(
            "  Query '{}': {:.2}μs avg, {} results",
            query,
            avg_time,
            results.len()
        );
    }
}

pub fn run_all_benchmarks() {
    println!("FF Performance Benchmarks");
    println!("=========================");

    benchmark_dataset_sizes();
    benchmark_query_types();
    benchmark_realistic_dataset();
    benchmark_large_dataset();
    benchmark_parallel_vs_sequential();

    println!("\n=== Benchmark Summary ===");
    println!("These benchmarks measure fuzzy filtering performance across different scenarios.");
    println!("Lower μs values indicate better performance.");
    println!("Parallel processing is automatically used for datasets > 1000 items.");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_benchmark_stats() {
        let duration = std::time::Duration::from_micros(1000);
        let iterations = 10;
        let avg_time = calculate_benchmark_stats(duration, iterations);
        assert_eq!(avg_time, 100.0);
    }

    #[test]
    fn test_format_benchmark_result() {
        let result = format_benchmark_result("test", 123.45, 5);
        assert_eq!(result, "  Query 'test': 123.45μs avg, 5 results");
    }

    #[test]
    fn test_get_benchmark_sizes() {
        let sizes = get_benchmark_sizes();
        assert_eq!(sizes, vec![100, 1000, 10000, 50000]);
    }

    #[test]
    fn test_get_benchmark_queries() {
        let queries = get_benchmark_queries();
        assert_eq!(queries, vec!["item", "item_1", "item_12", "item_123"]);
    }

    #[test]
    fn test_get_file_path_queries() {
        let queries = get_file_path_queries();
        assert_eq!(queries.len(), 10);
        assert!(queries.contains(&"src"));
        assert!(queries.contains(&"main"));
        assert!(queries.contains(&"rs"));
    }

    #[test]
    fn test_get_realistic_queries() {
        let queries = get_realistic_queries();
        assert_eq!(queries, vec!["src", "main", "rs", "test", "doc", "target"]);
    }

    #[test]
    fn test_get_large_dataset_queries() {
        let queries = get_large_dataset_queries();
        assert_eq!(
            queries,
            vec!["item", "item_1", "item_12", "item_123", "item_1234"]
        );
    }

    #[test]
    fn test_get_parallel_vs_sequential_sizes() {
        let sizes = get_parallel_vs_sequential_sizes();
        assert_eq!(sizes, vec![1000, 5000, 10000, 50000]);
    }

    #[test]
    fn test_get_parallel_vs_sequential_queries() {
        let queries = get_parallel_vs_sequential_queries();
        assert_eq!(queries, vec!["item", "item_1", "item_12"]);
    }

    #[test]
    fn test_generate_sequential_items() {
        let items = generate_sequential_items(3);
        assert_eq!(items, vec!["item_00000", "item_00001", "item_00002"]);
    }

    #[test]
    fn test_generate_sequential_items_empty() {
        let items = generate_sequential_items(0);
        assert_eq!(items, Vec::<String>::new());
    }

    #[test]
    fn test_generate_file_paths() {
        let paths = generate_file_paths(3);
        assert_eq!(paths.len(), 3);
        assert!(paths[0].contains("/"));
        assert!(paths[0].contains("."));
    }

    #[test]
    fn test_generate_file_paths_empty() {
        let paths = generate_file_paths(0);
        assert_eq!(paths, Vec::<String>::new());
    }

    #[test]
    fn test_generate_realistic_dataset() {
        let dataset = generate_realistic_dataset();
        assert_eq!(dataset.len(), 17);
        assert!(dataset.contains(&"src/main.rs".to_string()));
        assert!(dataset.contains(&"Cargo.toml".to_string()));
        assert!(dataset.contains(&"README.md".to_string()));
    }

    #[test]
    fn test_run_single_benchmark() {
        let items = vec!["test1".to_string(), "test2".to_string()];
        let (avg_time, results) = run_single_benchmark(items, "test", 1);

        assert!(avg_time >= 0.0);
        assert_eq!(results.len(), 2); // Both items should match "test"
    }

    #[test]
    fn test_run_single_benchmark_no_matches() {
        let items = vec!["abc".to_string(), "def".to_string()];
        let (avg_time, results) = run_single_benchmark(items, "xyz", 1);

        assert!(avg_time >= 0.0);
        assert_eq!(results.len(), 0); // No items should match "xyz"
    }

    #[test]
    fn test_benchmark_filtering() {
        let items = vec!["test1".to_string(), "test2".to_string()];
        let (avg_time, results) = benchmark_filtering(items, "test", 1);

        assert!(avg_time >= 0.0);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_benchmark_filtering_empty_query() {
        let items = vec!["test1".to_string(), "test2".to_string()];
        let (avg_time, results) = benchmark_filtering(items, "", 1);

        assert!(avg_time >= 0.0);
        assert_eq!(results.len(), 2); // Empty query should match all items
    }

    #[test]
    fn test_benchmark_filtering_no_matches() {
        let items = vec!["abc".to_string(), "def".to_string()];
        let (avg_time, results) = benchmark_filtering(items, "xyz", 1);

        assert!(avg_time >= 0.0);
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_benchmark_filtering_multiple_iterations() {
        let items = vec!["test1".to_string(), "test2".to_string()];
        let (avg_time, results) = benchmark_filtering(items, "test", 5);

        assert!(avg_time >= 0.0);
        assert_eq!(results.len(), 2);
    }
}
