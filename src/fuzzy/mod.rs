pub mod finder;
pub mod scoring;
pub mod stream;

pub use finder::{FuzzyFinder, MatchPositions};
pub use scoring::{MatchResult, score_batch, score_match, score_match_case_insensitive};
pub use stream::ItemStream;
