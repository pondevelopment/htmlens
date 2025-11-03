use url::Url;

/// Normalize a URL to its origin (scheme + host + optional port).
///
/// Falls back to trimming trailing slashes if the input cannot be parsed.
pub fn normalize_origin(input: &str) -> String {
    match Url::parse(input) {
        Ok(parsed) => parsed
            .origin()
            .ascii_serialization()
            .trim_end_matches('/')
            .to_string(),
        Err(_) => input.trim_end_matches('/').to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_standard_url() {
        let url = "https://example.com/path/page?query=true";
        assert_eq!(normalize_origin(url), "https://example.com");
    }

    #[test]
    fn keeps_port_information() {
        let url = "https://example.com:8443/path";
        assert_eq!(normalize_origin(url), "https://example.com:8443");
    }

    #[test]
    fn trims_trailing_slash_when_parse_fails() {
        let url = "example.com/";
        assert_eq!(normalize_origin(url), "example.com");
    }
}
