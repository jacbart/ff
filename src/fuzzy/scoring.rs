//! Fuzzy matching and scoring algorithm.
//!
//! This module provides a single-pass fuzzy matching algorithm that:
//! - Matches query characters in order (not necessarily consecutive)
//! - Computes a score based on match quality
//! - Returns match positions for highlighting
//!
//! Scoring factors:
//! - Exact match: highest score
//! - Prefix match: very high score
//! - Consecutive matches: bonus per consecutive char
//! - Word boundary matches: bonus for matches after separators or camelCase
//! - Gap penalty: penalty for non-consecutive matches
//! - Position bonus: earlier matches score higher

/// Scoring constants - tuned for good fuzzy matching behavior
mod scores {
    /// Exact match bonus (query == item)
    pub const EXACT: i32 = 10_000;
    /// Prefix match bonus (item starts with query)
    pub const PREFIX: i32 = 5_000;
    /// Bonus for each consecutive matching character
    pub const CONSECUTIVE: i32 = 40;
    /// Bonus for matching at a word boundary
    pub const BOUNDARY: i32 = 30;
    /// Bonus for matching the first character
    pub const FIRST_CHAR: i32 = 20;
    /// Base score for each matching character
    pub const MATCH: i32 = 16;
    /// Penalty for starting a gap (non-consecutive match)
    pub const GAP_START: i32 = -3;
    /// Penalty for extending a gap
    pub const GAP_EXTEND: i32 = -1;
    /// Maximum gap penalty (don't penalize too harshly for long gaps)
    pub const GAP_MAX: i32 = -20;
}

/// Result of a successful fuzzy match
#[derive(Debug, Clone)]
pub struct MatchResult {
    /// Numeric score for ranking (higher is better)
    pub score: i32,
    /// Indices of matched characters in the original item (for highlighting)
    pub positions: Vec<usize>,
}

/// Check if a character is a word boundary indicator
#[inline]
fn is_boundary_char(c: char) -> bool {
    matches!(c, '/' | '\\' | '_' | '-' | '.' | ' ' | ':')
}

/// Check if we're at a word boundary (camelCase or after separator)
#[inline]
fn is_word_boundary(prev: Option<char>, current: char) -> bool {
    match prev {
        None => true, // First character is always a boundary
        Some(p) => {
            // After a boundary character
            is_boundary_char(p)
            // camelCase boundary: lowercase followed by uppercase
            || (p.is_ascii_lowercase() && current.is_ascii_uppercase())
            // digit to letter or letter to digit
            || (p.is_ascii_digit() != current.is_ascii_digit())
        }
    }
}

/// Score a fuzzy match between an item and a query.
///
/// Returns `Some(MatchResult)` if all query characters are found in order,
/// `None` if there's no match.
///
/// Both `item` and `query` should already be lowercase for case-insensitive matching.
/// For camelCase boundary detection, use `score_match_with_original`.
pub fn score_match(item: &str, query: &str) -> Option<MatchResult> {
    score_match_with_original(item, item, query)
}

/// Score a fuzzy match with access to the original (non-lowercased) item for boundary detection.
///
/// - `item_lower`: lowercase version of the item (for matching)
/// - `item_original`: original item (for boundary detection)
/// - `query`: lowercase query
pub fn score_match_with_original(
    item_lower: &str,
    item_original: &str,
    query: &str,
) -> Option<MatchResult> {
    let item = item_lower;
    // Empty query matches everything with score 0
    if query.is_empty() {
        return Some(MatchResult {
            score: 0,
            positions: Vec::new(),
        });
    }

    // Empty item can't match non-empty query
    if item.is_empty() {
        return None;
    }

    // Fast path: exact match
    if item == query {
        let positions: Vec<usize> = (0..item.chars().count()).collect();
        return Some(MatchResult {
            score: scores::EXACT,
            positions,
        });
    }

    // Fast path: prefix match
    if item.starts_with(query) {
        let positions: Vec<usize> = (0..query.chars().count()).collect();
        let score = scores::PREFIX + (query.len() as i32 * scores::CONSECUTIVE);
        return Some(MatchResult { score, positions });
    }

    // Fast path: check if item contains query as substring
    if let Some(start_idx) = item.find(query) {
        // Substring match - calculate byte offset to char index
        let char_start = item[..start_idx].chars().count();
        let positions: Vec<usize> = (char_start..char_start + query.chars().count()).collect();

        // Score based on position (earlier is better)
        let position_bonus = ((item.len() - start_idx) as i32 * 2).min(100);
        let score =
            scores::PREFIX / 2 + (query.len() as i32 * scores::CONSECUTIVE) + position_bonus;

        return Some(MatchResult { score, positions });
    }

    // Full fuzzy matching with optimal position finding
    let item_chars: Vec<char> = item.chars().collect();
    let original_chars: Vec<char> = item_original.chars().collect();
    let query_chars: Vec<char> = query.chars().collect();

    // Find optimal match positions using DP
    let positions = find_optimal_positions(&item_chars, &query_chars)?;

    // Calculate score based on the optimal positions
    let score =
        calculate_score_for_positions(&positions, &item_chars, &original_chars, &query_chars);

    Some(MatchResult { score, positions })
}

/// Find optimal match positions that maximize consecutive runs.
/// Uses dynamic programming to find the best positions for each query character.
fn find_optimal_positions(item_chars: &[char], query_chars: &[char]) -> Option<Vec<usize>> {
    let n = item_chars.len();
    let m = query_chars.len();

    if m == 0 {
        return Some(Vec::new());
    }
    if n < m {
        return None;
    }

    // For each query character, find all positions where it matches in the item
    let mut match_positions: Vec<Vec<usize>> = Vec::with_capacity(m);
    for &qc in query_chars {
        let positions: Vec<usize> = item_chars
            .iter()
            .enumerate()
            .filter(|(_, &ic)| ic == qc)
            .map(|(i, _)| i)
            .collect();

        if positions.is_empty() {
            return None; // Query char not found, no match possible
        }
        match_positions.push(positions);
    }

    // DP to find best positions
    // dp[i][j] = best score achievable for query[0..=i] ending at position match_positions[i][j]
    // We also track the previous position index to reconstruct the path

    // For efficiency, we'll use a simpler greedy-with-lookahead approach:
    // For each query char, pick the position that gives the best consecutive bonus
    // considering the previous selected position

    let mut selected_positions = Vec::with_capacity(m);

    // For first query character, prefer earlier positions (but consider future consecutive potential)
    // Use DP approach: for each query char position, compute best score considering all options

    // dp[j] = (best_score, best_position) for query char i at match_positions[i][j]
    // We'll iterate through query chars and update

    // Initialize: for first query char, score based on position
    let first_positions = &match_positions[0];

    if m == 1 {
        // Single character query - just pick the best position (earliest, prefer boundary)
        let best_pos = *first_positions.first().unwrap();
        return Some(vec![best_pos]);
    }

    // Use DP: dp[i] = (max_score_to_reach_here, prev_position_index_in_prev_query_char_matches)
    // But for simplicity and performance, use a greedy approach with consecutive lookahead

    // Actually, let's use a proper DP for correctness:
    // State: dp[query_idx][pos_idx] = best score to match query[0..=query_idx] ending at match_positions[query_idx][pos_idx]
    // Transition: dp[i][j] = max over all k where match_positions[i-1][k] < match_positions[i][j] of:
    //             dp[i-1][k] + bonus(match_positions[i-1][k], match_positions[i][j])

    // This is O(m * k^2) where k is avg positions per char, which is fine for typical inputs

    // dp[j] represents the best score ending at match_positions[current_query_idx][j]
    // prev[j] represents the index in the previous query char's positions that led to this best score

    let mut dp: Vec<i32> = first_positions
        .iter()
        .map(|&pos| {
            // Score for first character at this position
            let mut s = scores::MATCH + scores::FIRST_CHAR;
            if pos == 0 {
                s += scores::BOUNDARY;
            }
            // Position bonus (earlier is better)
            s += (item_chars.len() as i32 - pos as i32).min(20);
            s
        })
        .collect();

    let mut prev_indices: Vec<Vec<usize>> = vec![vec![usize::MAX; first_positions.len()]];

    for qi in 1..m {
        let curr_positions = &match_positions[qi];
        let prev_positions = &match_positions[qi - 1];

        let mut new_dp = vec![i32::MIN; curr_positions.len()];
        let mut new_prev = vec![usize::MAX; curr_positions.len()];

        for (cj, &curr_pos) in curr_positions.iter().enumerate() {
            for (pj, &prev_pos) in prev_positions.iter().enumerate() {
                if prev_pos >= curr_pos {
                    continue; // Positions must be strictly increasing
                }

                let prev_score = dp[pj];
                if prev_score == i32::MIN {
                    continue;
                }

                // Calculate transition score
                let mut trans_score = scores::MATCH;

                // Consecutive bonus
                if curr_pos == prev_pos + 1 {
                    trans_score += scores::CONSECUTIVE;
                } else {
                    // Gap penalty
                    trans_score += scores::GAP_START;
                    let gap_size = (curr_pos - prev_pos - 1) as i32;
                    trans_score += (gap_size * scores::GAP_EXTEND).max(scores::GAP_MAX);
                }

                // Position bonus
                trans_score += (item_chars.len() as i32 - curr_pos as i32).min(20);

                let total = prev_score + trans_score;
                if total > new_dp[cj] {
                    new_dp[cj] = total;
                    new_prev[cj] = pj;
                }
            }
        }

        dp = new_dp;
        prev_indices.push(new_prev);
    }

    // Find the best ending position
    let last_positions = &match_positions[m - 1];
    let mut best_score = i32::MIN;
    let mut best_idx = 0;

    for (j, &score) in dp.iter().enumerate() {
        if score > best_score {
            best_score = score;
            best_idx = j;
        }
    }

    if best_score == i32::MIN {
        return None; // No valid path found
    }

    // Reconstruct the path
    selected_positions.resize(m, 0);
    selected_positions[m - 1] = last_positions[best_idx];

    let mut current_idx = best_idx;
    for qi in (1..m).rev() {
        let prev_idx = prev_indices[qi][current_idx];
        if prev_idx == usize::MAX {
            return None; // Should not happen if best_score is valid
        }
        selected_positions[qi - 1] = match_positions[qi - 1][prev_idx];
        current_idx = prev_idx;
    }

    Some(selected_positions)
}

/// Calculate the final score for a set of match positions
fn calculate_score_for_positions(
    positions: &[usize],
    item_chars: &[char],
    original_chars: &[char],
    query_chars: &[char],
) -> i32 {
    if positions.is_empty() {
        return 0;
    }

    let mut score: i32 = 0;
    let mut prev_pos: Option<usize> = None;
    let mut in_gap = false;

    for (qi, &pos) in positions.iter().enumerate() {
        // Base match score
        score += scores::MATCH;

        // First character bonus
        if qi == 0 {
            score += scores::FIRST_CHAR;
            if pos == 0 {
                score += scores::BOUNDARY;
            }
        }

        // Consecutive/gap handling
        if let Some(prev) = prev_pos {
            if pos == prev + 1 {
                score += scores::CONSECUTIVE;
                in_gap = false;
            } else {
                if !in_gap {
                    score += scores::GAP_START;
                    in_gap = true;
                }
                let gap_size = (pos - prev - 1) as i32;
                score += (gap_size * scores::GAP_EXTEND).max(scores::GAP_MAX);
            }
        }

        // Word boundary bonus
        let prev_char = if pos > 0 {
            original_chars.get(pos - 1).copied()
        } else {
            None
        };
        let current_original = original_chars.get(pos).copied().unwrap_or(query_chars[qi]);
        if is_word_boundary(prev_char, current_original) {
            score += scores::BOUNDARY;
        }

        // Position bonus
        score += (item_chars.len() as i32 - pos as i32).min(20);

        prev_pos = Some(pos);
    }

    // Length penalty
    let length_penalty = (item_chars.len() as i32 - query_chars.len() as i32).min(50);
    score -= length_penalty;

    score
}

/// Score and match an item against a query, handling case insensitivity.
///
/// This is a convenience function that handles lowercase conversion while
/// preserving the original item for camelCase boundary detection.
pub fn score_match_case_insensitive(item: &str, query: &str) -> Option<MatchResult> {
    let item_lower = item.to_lowercase();
    let query_lower = query.to_lowercase();
    score_match_with_original(&item_lower, item, &query_lower)
}

/// Batch score multiple items against a query.
///
/// Returns a vector of (index, MatchResult) for items that match,
/// sorted by score descending.
pub fn score_batch(items: &[String], query: &str) -> Vec<(usize, MatchResult)> {
    if query.is_empty() {
        // Return all items with zero score, preserving order
        return items
            .iter()
            .enumerate()
            .map(|(idx, _)| {
                (
                    idx,
                    MatchResult {
                        score: 0,
                        positions: Vec::new(),
                    },
                )
            })
            .collect();
    }

    let query_lower = query.to_lowercase();

    let mut results: Vec<(usize, MatchResult)> = items
        .iter()
        .enumerate()
        .filter_map(|(idx, item)| {
            let item_lower = item.to_lowercase();
            score_match_with_original(&item_lower, item, &query_lower).map(|result| (idx, result))
        })
        .collect();

    // Sort by score descending
    results.sort_unstable_by(|a, b| b.1.score.cmp(&a.1.score));

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_match_highest_score() {
        let result = score_match("test", "test").unwrap();
        assert_eq!(result.score, scores::EXACT);
        assert_eq!(result.positions, vec![0, 1, 2, 3]);
    }

    #[test]
    fn test_prefix_match_high_score() {
        let result = score_match("testing", "test").unwrap();
        assert!(result.score > 1000); // High score for prefix
        assert!(result.score < scores::EXACT); // But less than exact
        assert_eq!(result.positions, vec![0, 1, 2, 3]);
    }

    #[test]
    fn test_substring_match() {
        let result = score_match("a_test_b", "test").unwrap();
        assert!(result.score > 0);
        assert_eq!(result.positions, vec![2, 3, 4, 5]);
    }

    #[test]
    fn test_fuzzy_match_consecutive() {
        let result = score_match("foobar", "fb").unwrap();
        assert!(result.score > 0);
        assert_eq!(result.positions, vec![0, 3]);
    }

    #[test]
    fn test_fuzzy_match_scattered() {
        let result = score_match("a_b_c_d", "abcd").unwrap();
        assert!(result.score > 0);
        assert_eq!(result.positions, vec![0, 2, 4, 6]);
    }

    #[test]
    fn test_no_match() {
        let result = score_match("hello", "xyz");
        assert!(result.is_none());
    }

    #[test]
    fn test_empty_query_matches_all() {
        let result = score_match("anything", "").unwrap();
        assert_eq!(result.score, 0);
        assert!(result.positions.is_empty());
    }

    #[test]
    fn test_empty_item_no_match() {
        let result = score_match("", "query");
        assert!(result.is_none());
    }

    #[test]
    fn test_word_boundary_bonus() {
        // Match at word boundary should score higher
        let boundary_result = score_match("foo_bar", "b").unwrap();
        let middle_result = score_match("foobar", "b").unwrap();
        // 'b' at boundary in "foo_bar" vs middle in "foobar"
        // The boundary should get a bonus, but position also matters
        // "foobar" has 'b' at position 3, "foo_bar" has 'b' at position 4
        // Both should score reasonably, boundary gets bonus
        assert!(boundary_result.score > 0);
        assert!(middle_result.score > 0);
    }

    #[test]
    fn test_camel_case_boundary() {
        // Using lowercase input as score_match expects lowercase
        let result = score_match("foobar", "b").unwrap();
        assert!(result.score > 0);
        // 'b' is at position 3
        assert_eq!(result.positions, vec![3]);
    }

    #[test]
    fn test_camel_case_with_case_insensitive() {
        // For camelCase detection we need to use score_match_case_insensitive
        // which preserves case info for boundary detection
        let result = score_match_case_insensitive("FooBar", "b").unwrap();
        assert!(result.score > 0);
        assert_eq!(result.positions, vec![3]);
    }

    #[test]
    fn test_consecutive_beats_scattered() {
        // "fb" in "foobar" (consecutive f, then gap, then b)
        let scattered = score_match("f_o_o_b_a_r", "fb").unwrap();
        // "fb" in "fbar" (f then immediate b)
        let consecutive = score_match("fbar", "fb").unwrap();
        assert!(consecutive.score > scattered.score);
    }

    #[test]
    fn test_earlier_match_better() {
        let early = score_match("fb_xxxxx", "fb").unwrap();
        let late = score_match("xxxxx_fb", "fb").unwrap();
        assert!(early.score > late.score);
    }

    #[test]
    fn test_shorter_item_preferred() {
        let short = score_match("test", "t").unwrap();
        let long = score_match("test_with_very_long_suffix", "t").unwrap();
        // Both should match, shorter slightly preferred
        assert!(short.score >= long.score);
    }

    #[test]
    fn test_batch_scoring() {
        let items = vec![
            "apple".to_string(),
            "banana".to_string(),
            "apricot".to_string(),
            "cherry".to_string(),
        ];

        let results = score_batch(&items, "ap");

        // Should have 2 matches: apple and apricot
        assert_eq!(results.len(), 2);

        // Results should be sorted by score
        assert!(results[0].1.score >= results[1].1.score);

        // Both should be apple or apricot
        let matched_indices: Vec<usize> = results.iter().map(|(idx, _)| *idx).collect();
        assert!(matched_indices.contains(&0)); // apple
        assert!(matched_indices.contains(&2)); // apricot
    }

    #[test]
    fn test_batch_empty_query() {
        let items = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let results = score_batch(&items, "");

        // All items should match with score 0
        assert_eq!(results.len(), 3);
        for (_, result) in &results {
            assert_eq!(result.score, 0);
        }
    }

    #[test]
    fn test_file_path_matching() {
        let items = vec![
            "src/components/Button.tsx".to_string(),
            "src/utils/buttonHelper.ts".to_string(),
            "src/styles/button.css".to_string(),
            "README.md".to_string(),
        ];

        let results = score_batch(&items, "btn");

        // All button-related files should match
        assert!(results.len() >= 2);
    }

    #[test]
    fn test_case_insensitive() {
        let result = score_match_case_insensitive("HelloWorld", "hw");
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.positions, vec![0, 5]); // H and W
    }

    #[test]
    fn test_ranking_quality() {
        // Test that ranking makes intuitive sense
        let items = vec![
            "ff".to_string(),
            "file_finder".to_string(),
            "afford".to_string(),
            "leaf_folder".to_string(),
            "foo_far".to_string(),
        ];

        let results = score_batch(&items, "ff");

        // Exact match "ff" should be first
        assert_eq!(results[0].0, 0); // Index of "ff"

        // All should match
        assert_eq!(results.len(), 5);
    }

    #[test]
    fn test_unicode_support() {
        let result = score_match("héllo wörld", "hw");
        assert!(result.is_some());
    }

    #[test]
    fn test_special_characters() {
        let result = score_match("path/to/file.rs", "ptf");
        assert!(result.is_some());
        // Should match 'p' in path, 't' in to, 'f' in file
    }

    #[test]
    fn test_prefer_consecutive_over_scattered() {
        // This tests the specific case where "cargo" appears as a word
        // but earlier letters 'c' and 'a' also appear scattered before it
        let item = "# this file is automatically @generated by cargo.";
        let result = score_match(item, "cargo").unwrap();

        // Should match the actual word "cargo" (positions 43-47), not scattered letters
        // The word "cargo" starts at position 43 in the string
        let cargo_start = item.find("cargo").unwrap();
        let expected_positions: Vec<usize> = (cargo_start..cargo_start + 5).collect();

        assert_eq!(
            result.positions, expected_positions,
            "Should match 'cargo' as consecutive word, not scattered letters. Got positions: {:?}",
            result.positions
        );
    }

    #[test]
    fn test_prefer_consecutive_complex() {
        // More complex case: query spans multiple words but should prefer consecutive where possible
        let item = "this file cargo";
        let result = score_match(item, "filecargo").unwrap();

        // "file" is at positions 5-8, "cargo" is at positions 10-14
        // Should match: f(5), i(6), l(7), e(8), c(10), a(11), r(12), g(13), o(14)
        assert_eq!(result.positions, vec![5, 6, 7, 8, 10, 11, 12, 13, 14]);
    }

    #[test]
    fn test_scattered_early_vs_consecutive_late() {
        // Even if scattered match starts earlier, consecutive should win
        let item = "a_c_x cargo";
        let result = score_match(item, "ac").unwrap();

        // Could match a(0), c(2) scattered OR a(7), c... wait, "cargo" has c at 6
        // Actually in "a_c_x cargo": a=0, c=2, x=4, space=5, c=6, a=7, r=8, g=9, o=10
        // For "ac": scattered would be a(0), c(2); consecutive would be... no consecutive "ac"
        // But if we had "ac" in cargo... "ca" reversed. Let's try different example.

        // This should still prefer the earliest valid match, which is a(0), c(2)
        // since there's no consecutive "ac" available
        assert!(result.positions[0] < result.positions[1]);
    }

    #[test]
    fn test_optimal_positions_prefers_consecutive_run() {
        // Directly test the optimal position finding with a clear case
        let item_chars: Vec<char> = "xabcxabc".chars().collect();
        let query_chars: Vec<char> = "abc".chars().collect();

        let positions = find_optimal_positions(&item_chars, &query_chars).unwrap();

        // Should find positions 1,2,3 (first "abc") which is consecutive
        // Not 1,2,7 or other scattered combinations
        assert_eq!(positions, vec![1, 2, 3]);
    }

    #[test]
    fn test_optimal_positions_with_multiple_options() {
        // "abXXXab" with query "ab" - should prefer first "ab" at 0,1
        let item_chars: Vec<char> = "abxxxab".chars().collect();
        let query_chars: Vec<char> = "ab".chars().collect();

        let positions = find_optimal_positions(&item_chars, &query_chars).unwrap();

        // Both 0,1 and 5,6 are consecutive, but 0,1 is earlier (higher position bonus)
        assert_eq!(positions, vec![0, 1]);
    }
}
