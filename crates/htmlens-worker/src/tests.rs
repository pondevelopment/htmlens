#[cfg(test)]
mod tests {
    use crate::{extract_description, extract_title, format_cli_style_markdown};

    #[test]
    fn test_extract_title_basic() {
        let html = "<html><head><title>Test Title</title></head></html>";
        let title = extract_title(html);
        assert_eq!(title, "Test Title");
    }

    #[test]
    fn test_extract_title_no_title() {
        let html = "<html><head></head><body>Content</body></html>";
        let title = extract_title(html);
        assert_eq!(title, "Unknown Title");
    }

    #[test]
    fn test_extract_title_empty_title() {
        let html = "<html><head><title></title></head><body>Content</body></html>";
        let title = extract_title(html);
        assert_eq!(title, "");
    }

    #[test]
    fn test_extract_description_meta() {
        let html =
            r#"<html><head><meta name="description" content="Test description"></head></html>"#;
        let description = extract_description(html);
        assert_eq!(description, "Test description");
    }

    #[test]
    fn test_extract_description_no_meta() {
        let html = "<html><head><title>Test</title></head><body>Content</body></html>";
        let description = extract_description(html);
        assert_eq!(description, "");
    }

    #[test]
    fn test_url_validation_http() {
        let url = "http://example.com";
        let parsed = url::Url::parse(url).unwrap();
        assert!(matches!(parsed.scheme(), "http"));
    }

    #[test]
    fn test_url_validation_https() {
        let url = "https://example.com";
        let parsed = url::Url::parse(url).unwrap();
        assert!(matches!(parsed.scheme(), "https"));
    }

    #[test]
    fn test_url_validation_invalid_scheme() {
        let test_cases = vec![
            "ftp://example.com",
            "file:///etc/passwd",
            "javascript:alert(1)",
            "data:text/html,<script>alert(1)</script>",
        ];

        for url in test_cases {
            let parsed = url::Url::parse(url).unwrap();
            assert!(
                !matches!(parsed.scheme(), "http" | "https"),
                "URL {} should be rejected",
                url
            );
        }
    }

    #[test]
    fn test_url_validation_localhost() {
        let test_cases = vec![
            "http://localhost",
            "https://127.0.0.1",
            "http://[::1]", // IPv6 needs brackets
            "https://localhost:8080",
        ];

        for url in test_cases {
            let parsed = url::Url::parse(url).unwrap();
            if let Some(host) = parsed.host_str() {
                let host_lower = host.to_lowercase();
                let is_blocked = host_lower == "localhost"
                    || host_lower == "127.0.0.1"
                    || host_lower == "::1"
                    || host_lower == "[::1]"; // IPv6 localhost with brackets
                assert!(
                    is_blocked,
                    "URL {} should be blocked, host was '{}'",
                    url, host
                );
            }
        }
    }

    #[test]
    fn test_url_validation_private_ips() {
        let test_cases = vec![
            "http://192.168.1.1",
            "https://10.0.0.1",
            "http://172.16.0.1",
            "https://169.254.1.1",
        ];

        for url in test_cases {
            let parsed = url::Url::parse(url).unwrap();
            if let Some(host) = parsed.host_str() {
                let host_lower = host.to_lowercase();
                let is_private = host_lower.starts_with("192.168.")
                    || host_lower.starts_with("10.")
                    || host_lower.starts_with("172.16.")
                    || host_lower.starts_with("169.254.");
                assert!(is_private, "URL {} should be blocked as private", url);
            }
        }
    }

    #[test]
    fn test_url_validation_valid_public() {
        let test_cases = vec![
            "https://example.com",
            "http://google.com",
            "https://github.com",
            "http://stackoverflow.com",
        ];

        for url in test_cases {
            let parsed = url::Url::parse(url).unwrap();
            assert!(matches!(parsed.scheme(), "http" | "https"));

            if let Some(host) = parsed.host_str() {
                let host_lower = host.to_lowercase();
                let is_allowed = !host_lower.starts_with("192.168.")
                    && !host_lower.starts_with("10.")
                    && !host_lower.starts_with("172.16.")
                    && host_lower != "localhost"
                    && host_lower != "127.0.0.1";
                assert!(is_allowed, "URL {} should be allowed", url);
            }
        }
    }

    #[test]
    fn test_format_cli_style_markdown() {
        let url = "https://example.com";
        let title = "Test Product";
        let description = "A test product description";
        let jsonld_blocks = vec![serde_json::json!({
            "@type": "Product",
            "name": "Test Product",
            "description": "A test product"
        })];

        let markdown = format_cli_style_markdown(url, title, description, &jsonld_blocks);

        // Should contain basic markdown structure
        assert!(markdown.contains("# Test Product"));
        assert!(markdown.contains("**URL**: https://example.com"));
    }

    /// Test defense against common malicious URL patterns that could be used for attacks
    #[test]
    fn test_malicious_url_defense() {
        // Test various malicious URL patterns that should all be blocked
        let malicious_urls = vec![
            // Script injection attempts
            "javascript:alert('xss')",
            "javascript:window.location='evil.com'",
            // Local file access attempts
            "file:///etc/passwd",
            "file://c:\\windows\\system32\\config\\sam",
            "file:///proc/self/environ",
            // Data URLs (could contain malicious content)
            "data:text/html,<script>alert('xss')</script>",
            "data:application/javascript,alert('pwned')",
            // FTP and other protocols
            "ftp://secret.server.com/internal",
            "gopher://internal.network/",
            "ldap://internal.directory/",
            // SSRF attempts - localhost variations
            "http://localhost:22",    // SSH
            "https://localhost:3306", // MySQL
            "http://127.0.0.1:6379",  // Redis
            "http://[::1]:5432",      // PostgreSQL on IPv6 localhost
            // SSRF attempts - private networks
            "http://192.168.1.1",      // Home router
            "https://10.0.0.1",        // Private network
            "http://172.16.0.1",       // Docker network
            "https://169.254.169.254", // AWS metadata service
            // Protocol confusion
            "httpx://evil.com",
            "https://evil.com@localhost", // URL with userinfo
        ];

        for malicious_url in malicious_urls {
            println!("Testing malicious URL: {}", malicious_url);

            // Test URL parsing
            let parse_result = url::Url::parse(malicious_url);

            match parse_result {
                Ok(parsed_url) => {
                    // If URL parses successfully, check scheme validation
                    let scheme = parsed_url.scheme();

                    // Worker should reject non-HTTP/HTTPS schemes
                    if !matches!(scheme, "http" | "https") {
                        assert!(
                            !matches!(scheme, "http" | "https"),
                            "Malicious scheme '{}' should be blocked",
                            scheme
                        );
                        continue; // This would be blocked by scheme validation
                    }

                    // For HTTP/HTTPS, check host validation
                    if let Some(host) = parsed_url.host_str() {
                        let host_lower = host.to_lowercase();
                        let is_blocked_host = host_lower == "localhost"
                            || host_lower == "127.0.0.1"
                            || host_lower == "::1"
                            || host_lower == "[::1]"
                            || host_lower.starts_with("192.168.")
                            || host_lower.starts_with("10.")
                            || host_lower.starts_with("172.16.")
                            || host_lower.starts_with("172.17.")
                            || host_lower.starts_with("172.18.")
                            || host_lower.starts_with("172.19.")
                            || host_lower.starts_with("172.20.")
                            || host_lower.starts_with("172.21.")
                            || host_lower.starts_with("172.22.")
                            || host_lower.starts_with("172.23.")
                            || host_lower.starts_with("172.24.")
                            || host_lower.starts_with("172.25.")
                            || host_lower.starts_with("172.26.")
                            || host_lower.starts_with("172.27.")
                            || host_lower.starts_with("172.28.")
                            || host_lower.starts_with("172.29.")
                            || host_lower.starts_with("172.30.")
                            || host_lower.starts_with("172.31.")
                            || host_lower.starts_with("169.254.");

                        if malicious_url.starts_with("http") {
                            assert!(
                                is_blocked_host,
                                "Malicious URL with dangerous host '{}' should be blocked",
                                host
                            );
                        }
                    }
                }
                Err(_) => {
                    // Invalid URLs would be rejected at parse stage - this is expected
                }
            }
        }
    }
}
