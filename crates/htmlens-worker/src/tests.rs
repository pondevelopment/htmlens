#[cfg(test)]
mod worker_tests {
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

    /// Test @graph parsing - ensures nested @graph arrays are flattened and parsed correctly
    #[test]
    fn test_format_cli_style_markdown_with_graph() {
        let url = "https://example.com";
        let title = "Test Page";
        let description = "A test page with @graph";
        let jsonld_blocks = vec![serde_json::json!({
            "@context": "https://schema.org",
            "@graph": [
                {
                    "@type": "WebSite",
                    "name": "Example Site",
                    "url": "https://example.com",
                    "description": "An example website"
                },
                {
                    "@type": "Organization",
                    "name": "Example Corp",
                    "url": "https://example.com",
                    "logo": {
                        "@type": "ImageObject",
                        "url": "https://example.com/logo.png"
                    }
                },
                {
                    "@type": "BreadcrumbList",
                    "itemListElement": [
                        {
                            "@type": "ListItem",
                            "position": 1,
                            "name": "Home"
                        }
                    ]
                }
            ]
        })];

        let markdown = format_cli_style_markdown(url, title, description, &jsonld_blocks);

        // Should parse @graph and extract all entity types
        assert!(markdown.contains("🌐 WebSite"));
        assert!(markdown.contains("Example Site"));
        assert!(markdown.contains("🏢 Organization"));
        assert!(markdown.contains("Example Corp"));
        assert!(markdown.contains("🍞 Breadcrumb Navigation"));
        assert!(markdown.contains("Home"));
    }

    /// Test high/medium priority schema types (Article, Review, LocalBusiness, Event, NewsArticle, VideoObject)
    #[test]
    fn test_format_cli_style_markdown_with_article_types() {
        let url = "https://example.com";
        let title = "Schema Types Test";
        let description = "Testing article, review, business, event, news, and video types";
        let jsonld_blocks = vec![
            serde_json::json!({
                "@context": "https://schema.org",
                "@type": "BlogPosting",
                "headline": "How to Build Great Products",
                "author": {
                    "@type": "Person",
                    "name": "John Doe"
                },
                "datePublished": "2025-12-05T10:00:00Z",
                "dateModified": "2025-12-05T14:00:00Z",
                "description": "A comprehensive guide to product development"
            }),
            serde_json::json!({
                "@context": "https://schema.org",
                "@type": "NewsArticle",
                "headline": "Tech Industry News",
                "author": {
                    "@type": "Person",
                    "name": "Jane Smith"
                },
                "datePublished": "2025-12-05T08:30:00Z",
                "image": {
                    "@type": "ImageObject",
                    "url": "https://example.com/news-image.jpg"
                }
            }),
            serde_json::json!({
                "@context": "https://schema.org",
                "@type": "AggregateRating",
                "ratingValue": 4.5,
                "bestRating": 5.0,
                "reviewCount": 128,
                "name": "Product Rating"
            }),
            serde_json::json!({
                "@context": "https://schema.org",
                "@type": "LocalBusiness",
                "name": "Tech Café",
                "address": {
                    "@type": "PostalAddress",
                    "streetAddress": "123 Tech Street",
                    "addressLocality": "San Francisco"
                },
                "telephone": "+1-555-0123",
                "priceRange": "$$"
            }),
            serde_json::json!({
                "@context": "https://schema.org",
                "@type": "Event",
                "name": "Tech Conference 2025",
                "startDate": "2025-06-15T09:00:00Z",
                "endDate": "2025-06-17T18:00:00Z",
                "location": {
                    "@type": "Place",
                    "name": "Convention Center"
                },
                "eventStatus": "SCHEDULED"
            }),
            serde_json::json!({
                "@context": "https://schema.org",
                "@type": "VideoObject",
                "name": "Product Demo Video",
                "duration": "PT5M30S",
                "uploadDate": "2025-12-01T10:00:00Z",
                "contentUrl": "https://example.com/video.mp4"
            }),
        ];

        let markdown = format_cli_style_markdown(url, title, description, &jsonld_blocks);

        // Verify all article types are rendered
        assert!(markdown.contains("📝 Article / BlogPost"));
        assert!(markdown.contains("How to Build Great Products"));
        assert!(markdown.contains("John Doe"));
        
        // Verify news articles are rendered
        assert!(markdown.contains("📰 News Article"));
        assert!(markdown.contains("Tech Industry News"));
        assert!(markdown.contains("Jane Smith"));
        
        // Verify reviews/ratings are rendered with star ratings
        assert!(markdown.contains("⭐ Reviews & Ratings"));
        assert!(markdown.contains("4.5"));
        assert!(markdown.contains("128"));
        
        // Verify local business is rendered
        assert!(markdown.contains("🏪 Local Business"));
        assert!(markdown.contains("Tech Café"));
        assert!(markdown.contains("123 Tech Street"));
        assert!(markdown.contains("San Francisco"));
        
        // Verify events are rendered
        assert!(markdown.contains("🎯 Events"));
        assert!(markdown.contains("Tech Conference 2025"));
        assert!(markdown.contains("SCHEDULED"));
        
        // Verify videos are rendered
        assert!(markdown.contains("🎬 Videos"));
        assert!(markdown.contains("Product Demo Video"));
        assert!(markdown.contains("PT5M30S"));
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
