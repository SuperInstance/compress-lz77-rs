//! Sliding window buffer for LZ77 back-reference management.

/// A sliding window over byte data that supports looking back for matches.
///
/// The window maintains a view into the input data with a fixed maximum size
/// for back-references. It is used during encoding to find repeated patterns.
#[derive(Debug)]
pub struct SlidingWindow {
    /// The full input data being encoded.
    data: Vec<u8>,
    /// Current position in the data.
    position: usize,
    /// Maximum distance to look back.
    max_distance: usize,
}

impl SlidingWindow {
    /// Create a new sliding window over the given data.
    ///
    /// # Arguments
    ///
    /// * `data` - The input data to slide over.
    /// * `max_distance` - Maximum lookback distance (window size).
    pub fn new(data: &[u8], max_distance: usize) -> Self {
        SlidingWindow {
            data: data.to_vec(),
            position: 0,
            max_distance,
        }
    }

    /// Advance the position by `n` bytes.
    pub fn advance(&mut self, n: usize) {
        self.position = (self.position + n).min(self.data.len());
    }

    /// Current position in the data.
    pub fn position(&self) -> usize {
        self.position
    }

    /// Total data length.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Whether the data is empty.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Whether we've consumed all data.
    pub fn is_exhausted(&self) -> bool {
        self.position >= self.data.len()
    }

    /// Bytes remaining to be processed.
    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.position)
    }

    /// Get the byte at the current position (returns `None` if exhausted).
    pub fn current_byte(&self) -> Option<u8> {
        self.data.get(self.position).copied()
    }

    /// Get a slice of the lookahead buffer (bytes from current position).
    pub fn lookahead(&self, max_len: usize) -> &[u8] {
        let end = (self.position + max_len).min(self.data.len());
        if self.position >= end {
            &[]
        } else {
            &self.data[self.position..end]
        }
    }

    /// Get the start of the search window (lookback start).
    ///
    /// Returns the position `max_distance` bytes back from the current position,
    /// clamped to 0.
    pub fn search_start(&self) -> usize {
        self.position.saturating_sub(self.max_distance)
    }

    /// Get the lookback window as a slice.
    pub fn lookback(&self) -> &[u8] {
        &self.data[self.search_start()..self.position]
    }

    /// Get a byte from the lookback window by distance (1 = previous byte).
    ///
    /// Returns `None` if the distance is out of range.
    pub fn byte_at_distance(&self, distance: usize) -> Option<u8> {
        if distance == 0 || distance > self.position || distance > self.max_distance {
            return None;
        }
        self.data.get(self.position - distance).copied()
    }

    /// Compare a run of bytes starting at a lookback offset with the lookahead.
    ///
    /// Returns the length of the match (0 if no match at all).
    /// The match can extend past the lookback into the lookahead (self-overlapping).
    pub fn match_length(&self, offset: usize, max_len: usize) -> usize {
        let search_start = self.search_start();
        if offset < search_start || offset >= self.position {
            return 0;
        }

        let lookahead_end = (self.position + max_len).min(self.data.len());
        let mut len = 0;

        // We compare byte by byte, allowing self-overlapping matches
        let mut src = offset;
        let mut dst = self.position;
        while dst < lookahead_end {
            if self.data[src] != self.data[dst] {
                break;
            }
            len += 1;
            src += 1;
            dst += 1;
            // Self-overlap: src wraps back to offset when it reaches current position
            if src >= self.position {
                src = offset;
            }
        }

        len
    }

    /// Access the raw underlying data.
    pub fn data(&self) -> &[u8] {
        &self.data
    }
}

impl Clone for SlidingWindow {
    fn clone(&self) -> Self {
        SlidingWindow {
            data: self.data.clone(),
            position: self.position,
            max_distance: self.max_distance,
        }
    }
}
