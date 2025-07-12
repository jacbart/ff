pub mod finder;
pub mod matching;

pub use finder::FuzzyFinder;
pub use matching::{fuzzy_match, sequential_filter, parallel_filter}; 