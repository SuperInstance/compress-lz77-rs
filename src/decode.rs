//! LZ77 decoding from token streams.

use crate::token::Token;

/// Decode a token stream back into the original byte data.
///
/// Supports self-overlapping references (e.g., a reference with length > distance
/// that produces runs like `AAAA`).
///
/// # Panics
///
/// Panics if a reference token has `distance > current_output_length`.
pub fn decode(tokens: &[Token]) -> Vec<u8> {
    let mut output = Vec::new();

    for token in tokens {
        match token {
            Token::Literal(b) => {
                output.push(*b);
            }
            Token::Reference { length, distance } => {
                let start = output.len() - *distance;
                assert!(
                    start <= output.len(),
                    "distance {} exceeds output length {}",
                    distance,
                    output.len()
                );
                // Copy byte-by-byte to support self-overlapping references
                for i in 0..*length {
                    let byte = output[start + i];
                    output.push(byte);
                }
            }
        }
    }

    output
}

/// Decode and verify that the output matches expected data.
///
/// Returns `Ok(output)` if decoding succeeds, `Err(output)` if the output
/// doesn't match the expected data.
pub fn decode_and_verify(tokens: &[Token], expected: &[u8]) -> Result<Vec<u8>, Vec<u8>> {
    let output = decode(tokens);
    if output == expected {
        Ok(output)
    } else {
        Err(output)
    }
}

/// Count the number of literal and reference tokens.
///
/// Returns `(literal_count, reference_count)`.
pub fn count_token_types(tokens: &[Token]) -> (usize, usize) {
    let literals = tokens.iter().filter(|t| t.is_literal()).count();
    (literals, tokens.len() - literals)
}

/// Calculate the total output length from a token stream without decoding.
pub fn output_length(tokens: &[Token]) -> usize {
    tokens.iter().map(|t| t.output_length()).sum()
}
