pub mod finder;
pub mod scoring;
pub mod stream;

pub use finder::{FuzzyFinder, MatchPositions};
pub use scoring::{score_batch, score_match, score_match_case_insensitive, MatchResult};
pub use stream::ItemStream;
