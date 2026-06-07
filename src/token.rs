//! Length-distance token representation for LZ77.

/// A single LZ77 token, representing either a literal byte or a back-reference.
///
/// # Back-references
///
/// A back-reference `(length, distance)` means "copy `length` bytes starting
/// from `distance` bytes back from the current output position." The copy can
/// overlap with the output (self-overlapping), which handles runs like `AAAA`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    /// A literal byte that could not be matched.
    Literal(u8),
    /// A back-reference: `(length, distance)`.
    ///
    /// - `length` — Number of bytes to copy (≥ `min_match`).
    /// - `distance` — How far back to look (≥ 1).
    Reference { length: usize, distance: usize },
}

impl Token {
    /// Create a literal token.
    pub fn literal(byte: u8) -> Self {
        Token::Literal(byte)
    }

    /// Create a reference token.
    pub fn reference(length: usize, distance: usize) -> Self {
        assert!(distance > 0, "distance must be > 0");
        Token::Reference { length, distance }
    }

    /// Whether this is a literal token.
    pub fn is_literal(&self) -> bool {
        matches!(self, Token::Literal(_))
    }

    /// Whether this is a reference token.
    pub fn is_reference(&self) -> bool {
        matches!(self, Token::Reference { .. })
    }

    /// If literal, return the byte value.
    pub fn literal_value(&self) -> Option<u8> {
        match self {
            Token::Literal(b) => Some(*b),
            _ => None,
        }
    }

    /// If reference, return (length, distance).
    pub fn reference_values(&self) -> Option<(usize, usize)> {
        match self {
            Token::Reference { length, distance } => Some((*length, *distance)),
            _ => None,
        }
    }

    /// The number of output bytes this token produces when decoded.
    pub fn output_length(&self) -> usize {
        match self {
            Token::Literal(_) => 1,
            Token::Reference { length, .. } => *length,
        }
    }
}

/// Serialize a token stream into a compact byte representation.
///
/// Format per token:
/// - Literal: `[0x00, byte]`
/// - Reference: `[length_hi | 0x80, length_lo, distance_hi, distance_lo]`
///
/// This uses 4 bytes per reference (supports length up to 65535, distance up to 65535).
pub fn serialize_tokens(tokens: &[Token]) -> Vec<u8> {
    let mut out = Vec::with_capacity(tokens.len() * 2);
    for token in tokens {
        match token {
            Token::Literal(b) => {
                out.push(0x00);
                out.push(*b);
            }
            Token::Reference { length, distance } => {
                let len_hi = (*length >> 8) as u8;
                let len_lo = (*length & 0xFF) as u8;
                let dist_hi = (*distance >> 8) as u8;
                let dist_lo = (*distance & 0xFF) as u8;
                out.push(len_hi | 0x80);
                out.push(len_lo);
                out.push(dist_hi);
                out.push(dist_lo);
            }
        }
    }
    out
}

/// Deserialize tokens from the compact byte representation produced by
/// [`serialize_tokens`].
///
/// Returns `None` if the input is malformed.
pub fn deserialize_tokens(data: &[u8]) -> Option<Vec<Token>> {
    let mut tokens = Vec::new();
    let mut i = 0;
    while i < data.len() {
        let header = data[i];
        if header & 0x80 == 0 {
            // Literal
            i += 1;
            if i >= data.len() {
                return None;
            }
            tokens.push(Token::Literal(data[i]));
            i += 1;
        } else {
            // Reference
            if i + 3 >= data.len() {
                return None;
            }
            let length = (((header & 0x7F) as usize) << 8) | (data[i + 1] as usize);
            let distance = ((data[i + 2] as usize) << 8) | (data[i + 3] as usize);
            if distance == 0 {
                return None;
            }
            tokens.push(Token::Reference { length, distance });
            i += 4;
        }
    }
    Some(tokens)
}
