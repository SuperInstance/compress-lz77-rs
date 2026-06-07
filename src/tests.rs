//! Comprehensive test suite for compress-lz77-rs.

#[cfg(test)]
mod tests {
    use crate::{encode, decode, token, window, match_};

    /// Helper: round-trip encode/decode with default parameters.
    fn round_trip(data: &[u8]) -> Vec<u8> {
        let tokens = encode::encode(data, 4096, 3, 258);
        decode::decode(&tokens)
    }

    /// Helper: round-trip with lazy matching.
    fn round_trip_lazy(data: &[u8]) -> Vec<u8> {
        let tokens = encode::encode_lazy(data, 4096, 3, 258);
        decode::decode(&tokens)
    }

    // ── Window tests ────────────────────────────────────────────────────

    #[test]
    fn test_window_basic() {
        let w = window::SlidingWindow::new(b"hello", 10);
        assert_eq!(w.position(), 0);
        assert_eq!(w.remaining(), 5);
        assert!(!w.is_exhausted());
        assert_eq!(w.current_byte(), Some(b'h'));
    }

    #[test]
    fn test_window_advance() {
        let mut w = window::SlidingWindow::new(b"hello", 10);
        w.advance(2);
        assert_eq!(w.position(), 2);
        assert_eq!(w.current_byte(), Some(b'l'));
    }

    #[test]
    fn test_window_exhausted() {
        let mut w = window::SlidingWindow::new(b"ab", 10);
        w.advance(5); // past end
        assert!(w.is_exhausted());
        assert_eq!(w.current_byte(), None);
    }

    #[test]
    fn test_window_lookahead() {
        let w = window::SlidingWindow::new(b"hello world", 10);
        assert_eq!(w.lookahead(5), b"hello");
    }

    #[test]
    fn test_window_lookback() {
        let mut w = window::SlidingWindow::new(b"hello", 10);
        w.advance(3);
        assert_eq!(w.lookback(), b"hel");
    }

    #[test]
    fn test_window_match_length() {
        let mut w = window::SlidingWindow::new(b"abcabc", 10);
        w.advance(3); // position at second "abc"
        let len = w.match_length(0, 10);
        assert_eq!(len, 3);
    }

    #[test]
    fn test_window_self_overlapping_match() {
        // "aaaa" - match at offset 0 should extend via self-overlap
        let mut w = window::SlidingWindow::new(b"aaaa", 10);
        w.advance(1); // position at index 1
        let len = w.match_length(0, 10);
        assert_eq!(len, 3); // can match 3 more 'a's
    }

    #[test]
    fn test_window_byte_at_distance() {
        let mut w = window::SlidingWindow::new(b"abcde", 10);
        w.advance(3);
        assert_eq!(w.byte_at_distance(1), Some(b'c'));
        assert_eq!(w.byte_at_distance(2), Some(b'b'));
        assert_eq!(w.byte_at_distance(0), None);
    }

    // ── Match finding tests ─────────────────────────────────────────────

    #[test]
    fn test_find_match_basic() {
        let mut w = window::SlidingWindow::new(b"abcabc", 10);
        w.advance(3);
        let m = match_::find_longest_match(&w, 2, 10);
        assert!(m.is_some());
        let m = m.unwrap();
        assert_eq!(m.distance, 3);
        assert_eq!(m.length, 3);
    }

    #[test]
    fn test_find_match_no_match() {
        let mut w = window::SlidingWindow::new(b"abcdef", 10);
        w.advance(3);
        let m = match_::find_longest_match(&w, 2, 10);
        assert!(m.is_none()); // "def" doesn't match "abc"
    }

    #[test]
    fn test_find_match_too_short() {
        let mut w = window::SlidingWindow::new(b"abca", 10);
        w.advance(3);
        // "a" matches but min_match is 2
        let m = match_::find_longest_match(&w, 2, 10);
        assert!(m.is_none());
    }

    #[test]
    fn test_find_nearest_match() {
        let mut w = window::SlidingWindow::new(b"abcabcabc", 10);
        w.advance(3);
        let m = match_::find_nearest_match(&w, 2, 10);
        assert!(m.is_some());
        // Should find the nearest (distance 3)
        assert_eq!(m.unwrap().distance, 3);
    }

    // ── Token tests ─────────────────────────────────────────────────────

    #[test]
    fn test_token_literal() {
        let t = token::Token::literal(b'x');
        assert!(t.is_literal());
        assert!(!t.is_reference());
        assert_eq!(t.literal_value(), Some(b'x'));
        assert_eq!(t.output_length(), 1);
    }

    #[test]
    fn test_token_reference() {
        let t = token::Token::reference(10, 5);
        assert!(t.is_reference());
        assert_eq!(t.reference_values(), Some((10, 5)));
        assert_eq!(t.output_length(), 10);
    }

    #[test]
    fn test_token_serialize_deserialize() {
        let tokens = vec![
            token::Token::literal(b'a'),
            token::Token::literal(b'b'),
            token::Token::reference(5, 3),
            token::Token::literal(b'x'),
        ];
        let serialized = token::serialize_tokens(&tokens);
        let deserialized = token::deserialize_tokens(&serialized).unwrap();
        assert_eq!(tokens, deserialized);
    }

    #[test]
    fn test_token_deserialize_malformed() {
        // Truncated reference
        assert!(token::deserialize_tokens(&[0x80]).is_none());
        assert!(token::deserialize_tokens(&[0x80, 0x01, 0x00]).is_none());
    }

    #[test]
    fn test_token_deserialize_empty() {
        let result = token::deserialize_tokens(&[]);
        assert_eq!(result, Some(vec![]));
    }

    // ── Encode/Decode round-trip tests ──────────────────────────────────

    #[test]
    fn test_roundtrip_simple() {
        assert_eq!(b"abracadabra".as_slice(), round_trip(b"abracadabra").as_slice());
    }

    #[test]
    fn test_roundtrip_repeated() {
        let data = b"abcabcabcabcabcabc";
        assert_eq!(data.as_slice(), round_trip(data).as_slice());
    }

    #[test]
    fn test_roundtrip_empty() {
        assert_eq!(Vec::<u8>::new(), round_trip(b""));
    }

    #[test]
    fn test_roundtrip_single_byte() {
        assert_eq!(b"x".as_slice(), round_trip(b"x").as_slice());
    }

    #[test]
    fn test_roundtrip_all_same() {
        let data = b"aaaaaaaaaaaaaaaaaaaa";
        assert_eq!(data.as_slice(), round_trip(data).as_slice());
    }

    #[test]
    fn test_roundtrip_no_repeats() {
        let data = b"abcdefghij";
        assert_eq!(data.as_slice(), round_trip(data).as_slice());
    }

    #[test]
    fn test_roundtrip_long_text() {
        let data = b"the quick brown fox jumps over the lazy dog. the quick brown fox jumps over the lazy dog again.";
        assert_eq!(data.as_slice(), round_trip(data).as_slice());
    }

    #[test]
    fn test_roundtrip_binary() {
        let data: Vec<u8> = (0..=255).cycle().take(512).collect();
        assert_eq!(data.as_slice(), round_trip(&data).as_slice());
    }

    #[test]
    fn test_roundtrip_lazy() {
        let data = b"abracadabra abracadabra";
        assert_eq!(data.as_slice(), round_trip_lazy(data).as_slice());
    }

    #[test]
    fn test_roundtrip_lazy_binary() {
        let data: Vec<u8> = (0..=255).collect();
        assert_eq!(data.as_slice(), round_trip_lazy(&data).as_slice());
    }

    // ── Compression ratio tests ─────────────────────────────────────────

    #[test]
    fn test_repetitive_data_compression_ratio() {
        let data = b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let tokens = encode::encode(data, 4096, 3, 258);
        // Should be mostly references
        let (literals, refs) = decode::count_token_types(&tokens);
        assert!(refs > 0, "expected some references for repetitive data");
        assert!(literals < data.len(), "should not be all literals");
    }

    #[test]
    fn test_compression_estimate() {
        let data = b"abcabcabc";
        let tokens = encode::encode(data, 4096, 3, 258);
        let (compressed, original) = encode::compression_estimate(&tokens);
        assert_eq!(original, data.len());
        assert!(compressed < original * 4, "compressed estimate should be reasonable");
    }

    #[test]
    fn test_decode_verify() {
        let data = b"hello hello";
        let tokens = encode::encode(data, 4096, 3, 258);
        let result = decode::decode_and_verify(&tokens, data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_output_length() {
        let data = b"abcabcabc";
        let tokens = encode::encode(data, 4096, 3, 258);
        let computed = decode::output_length(&tokens);
        assert_eq!(computed, data.len());
    }

    #[test]
    fn test_small_window() {
        let data = b"abcabcabc";
        let tokens = encode::encode(data, 4, 3, 258);
        let decoded = decode::decode(&tokens);
        assert_eq!(data.as_slice(), decoded.as_slice());
    }

    #[test]
    fn test_window_boundary() {
        // Data longer than window size
        let data: Vec<u8> = (b'a'..=b'z').cycle().take(100).collect();
        let tokens = encode::encode(&data, 10, 3, 258);
        let decoded = decode::decode(&tokens);
        assert_eq!(data.as_slice(), decoded.as_slice());
    }
}
