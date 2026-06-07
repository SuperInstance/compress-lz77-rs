//! LZ77 encoding with optional lazy matching.

use crate::match_;
use crate::token::Token;
use crate::window::SlidingWindow;

/// Encode data using LZ77 with the specified parameters.
///
/// # Arguments
///
/// * `data` - Input data to compress.
/// * `window_size` - Maximum lookback distance.
/// * `min_match` - Minimum match length to emit a reference (typically 3).
/// * `max_match` - Maximum match length (typically 258).
///
/// # Returns
///
/// A vector of [`Token`]s representing the compressed data.
pub fn encode(data: &[u8], window_size: usize, min_match: usize, max_match: usize) -> Vec<Token> {
    encode_with_strategy(data, window_size, min_match, max_match, Strategy::Greedy)
}

/// Encode using lazy matching strategy.
///
/// Lazy matching checks if the next position has a better match before
/// committing to the current one. This often produces better compression.
pub fn encode_lazy(
    data: &[u8],
    window_size: usize,
    min_match: usize,
    max_match: usize,
) -> Vec<Token> {
    encode_with_strategy(data, window_size, min_match, max_match, Strategy::Lazy)
}

/// Encoding strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Strategy {
    Greedy,
    Lazy,
}

fn encode_with_strategy(
    data: &[u8],
    window_size: usize,
    min_match: usize,
    max_match: usize,
    strategy: Strategy,
) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut window = SlidingWindow::new(data, window_size);

    while !window.is_exhausted() {
        let (opt_match, should_emit_literal) = match strategy {
            Strategy::Greedy => {
                let m = match_::find_longest_match(&window, min_match, max_match);
                (m, false)
            }
            Strategy::Lazy => {
                let (m, lazy) = match_::find_lazy_match(&window, min_match, max_match);
                (m, lazy)
            }
        };

        match opt_match {
            Some(mr) if should_emit_literal => {
                // Emit current byte as literal, use lazy match next iteration
                tokens.push(Token::Literal(window.current_byte().unwrap()));
                window.advance(1);

                // Apply the lazy match
                tokens.push(Token::Reference {
                    length: mr.length,
                    distance: mr.distance,
                });
                window.advance(mr.length);
            }
            Some(mr) => {
                tokens.push(Token::Reference {
                    length: mr.length,
                    distance: mr.distance,
                });
                window.advance(mr.length);
            }
            None => {
                tokens.push(Token::Literal(window.current_byte().unwrap()));
                window.advance(1);
            }
        }
    }

    tokens
}

/// Compute a simple compression ratio metric.
///
/// Returns `(compressed_cost, original_size)` where compressed_cost is an
/// estimate of the token stream size in bytes.
pub fn compression_estimate(tokens: &[Token]) -> (usize, usize) {
    let original: usize = tokens.iter().map(|t| t.output_length()).sum();
    // Rough estimate: 1 byte per literal, 4 bytes per reference
    let compressed: usize = tokens
        .iter()
        .map(|t| if t.is_literal() { 2 } else { 4 })
        .sum();
    (compressed, original)
}
