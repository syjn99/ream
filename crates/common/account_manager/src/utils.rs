/// Validates that a string contains only valid hexadecimal characters
///
/// # Arguments
/// * `hex_str` - The string to validate
///
/// # Returns
/// * `bool` - true if valid, false if invalid
pub fn validate_hex_string(hex_str: &str) -> bool {
    hex_str.chars().all(|c| c.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_hex_string_valid() {
        assert!(validate_hex_string("0123456789abcdef"));
        assert!(validate_hex_string("ABCDEF"));
        assert!(validate_hex_string(""));
    }

    #[test]
    fn test_validate_hex_string_invalid() {
        assert!(!validate_hex_string("0123456789abcdefg"));
        assert!(!validate_hex_string("hello world"));
        assert!(!validate_hex_string("0123456789abcdef!"));
    }
}
