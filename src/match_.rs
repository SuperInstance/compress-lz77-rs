//! Match finding with configurable search parameters.

use crate::window::SlidingWindow;

/// The result of a match search: distance back and length of the match.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MatchResult {
    /// Distance back from the current position.
    pub distance: usize,
    /// Length of the match.
    pub length: usize,
}

impl MatchResult {
    /// Create a new match result.
    pub fn new(distance: usize, length: usize) -> Self {
        MatchResult { distance, length }
    }

    /// Whether this is a valid (non-empty) match.
    pub fn is_valid(&self) -> bool {
        self.distance > 0 && self.length > 0
    }
}

/// Find the longest match in the sliding window using brute-force search.
///
/// Searches the entire lookback window for the longest match starting at the
/// current position. Returns `None` if no match of at least `min_length` is found.
///
/// # Arguments
///
/// * `window` - The sliding window to search in.
/// * `min_length` - Minimum match length to consider.
/// * `max_length` - Maximum match length to report.
pub fn find_longest_match(
    window: &SlidingWindow,
    min_length: usize,
    max_length: usize,
) -> Option<MatchResult> {
    if window.is_exhausted() {
        return None;
    }

    let search_start = window.search_start();
    let pos = window.position();
    let mut best: Option<MatchResult> = None;

    for offset in (search_start..pos).rev() {
        let len = window.match_length(offset, max_length);
        if len >= min_length {
            let distance = pos - offset;
            match best {
                Some(b) if len > b.length => {
                    best = Some(MatchResult::new(distance, len));
                }
                None => {
                    best = Some(MatchResult::new(distance, len));
                }
                _ => {}
            }
            // If we hit max length, no need to search further
            if len >= max_length {
                break;
            }
        }
    }

    best
}

/// Find a match using lazy matching strategy.
///
/// First finds the best match at the current position, then checks if starting
/// one byte later yields a better (longer) match. If so, emits a literal and
/// uses the later match instead.
///
/// Returns `(best_match, should_emit_literal)` where `should_emit_literal` is
/// true when lazy matching determined the next position has a better match.
pub fn find_lazy_match(
    window: &SlidingWindow,
    min_length: usize,
    max_length: usize,
) -> (Option<MatchResult>, bool) {
    let current_match = find_longest_match(window, min_length, max_length);

    match current_match {
        Some(cm) => {
            // Check next position
            if window.remaining() > 1 {
                let mut next_window = window.clone();
                next_window.advance(1);
                let next_match = find_longest_match(&next_window, min_length, max_length);

                if let Some(nm) = next_match {
                    if nm.length > cm.length {
                        // Lazy: skip this match, emit literal, use next match
                        return (Some(nm), true);
                    }
                }
            }
            (Some(cm), false)
        }
        None => (None, false),
    }
}

/// Find all matches in a hash-chain style (simplified: returns the nearest match).
///
/// This is a faster but less optimal match finder that returns the first match
/// found (closest to current position) that meets the minimum length requirement.
pub fn find_nearest_match(
    window: &SlidingWindow,
    min_length: usize,
    max_length: usize,
) -> Option<MatchResult> {
    if window.is_exhausted() {
        return None;
    }

    let search_start = window.search_start();
    let pos = window.position();

    for offset in (search_start..pos).rev() {
        let len = window.match_length(offset, max_length);
        if len >= min_length {
            return Some(MatchResult::new(pos - offset, len));
        }
    }

    None
}
