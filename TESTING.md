# HTMLens Testing Guide

This document describes the comprehensive test suite for the HTMLens project.

## Test Structure

The HTMLens workspace includes tests at multiple levels:

### 1. Unit Tests (crates/htmlens-core/src/)
- **lib.rs**: Core type creation and basic functionality
- **parser.rs**: JSON-LD extraction, HTML sanitization, markdown conversion
- Tests run with: `cargo test --package htmlens-core`

### 2. CLI Tests (crates/htmlens-cli/)
- **src/lib.rs**: Utility function tests (shorten_iri, format_price, etc.)
- **tests/integration.rs**: End-to-end CLI functionality tests
- Tests run with: `cargo test --package htmlens-cli`

### 3. Worker Tests (crates/htmlens-worker/src/)
- **tests.rs**: URL validation, security checks, API response formatting
- **lib.rs**: Integration tests for worker functionality
- Tests run with: `cargo check --package htmlens-worker` (WASM environment needed for full tests)

### 4. Test Fixtures (tests/fixtures/)
- **product.html**: Simple product page with JSON-LD
- **product_group.html**: Complex product group with variants  
- **no_jsonld.html**: Page without structured data
- **direct_product.json**: Direct JSON-LD input for CLI testing

## Running Tests

### Quick Test (Recommended)
```bash
# Run all unit tests
cargo test --workspace

# Run specific package tests
cargo test --package htmlens-core
cargo test --package htmlens-cli
```

### Comprehensive Test Suite
```bash
# Use the test runner script
./test_runner.sh
```

### Individual Test Categories

#### Core Library Tests
```bash
# Default features (lightweight)
cargo test --package htmlens-core

# Full-expansion features 
cargo test --package htmlens-core --features full-expansion
```

#### CLI Tests
```bash
# Unit tests
cargo test --package htmlens-cli --lib

# Integration tests
cargo test --package htmlens-cli --test integration
```

#### Manual CLI Testing
```bash
# Test help output
cargo run --package htmlens-cli -- --help

# Test version
cargo run --package htmlens-cli -- --version

# Test with direct JSON-LD
cargo run --package htmlens-cli -- '{"@context": "https://schema.org", "@type": "Product", "name": "Test"}'

# Test with fixture file
cargo run --package htmlens-cli -- "$(cat tests/fixtures/direct_product.json)"
```

## Test Coverage

### Core Parser Functions ✅
- [x] JSON-LD extraction from HTML
- [x] Multiple JSON-LD blocks combination
- [x] HTML sanitization (removes scripts, styles, comments)
- [x] HTML to Markdown conversion
- [x] Empty and invalid input handling

### CLI Functionality ✅  
- [x] Help and version output
- [x] Direct JSON-LD processing
- [x] URL input validation
- [x] Output mode flags (-g, -G, -m)
- [x] Product group and variant analysis
- [x] Error handling for invalid inputs

### Worker Security ✅
- [x] URL scheme validation (HTTP/HTTPS only)
- [x] SSRF protection (blocks localhost, private IPs)
- [x] Input sanitization
- [x] API response formatting

### Integration Tests ✅
- [x] End-to-end CLI workflows
- [x] Complex product data processing
- [x] Error handling and edge cases
- [x] Output format validation

## Test Data

### Simple Product (tests/fixtures/product.html)
```json
{
  "@type": "Product",
  "name": "Test Product", 
  "price": "$29.99",
  "availability": "InStock"
}
```

### Product Group (tests/fixtures/product_group.html) 
```json
{
  "@type": "ProductGroup",
  "name": "Mountain Bike Collection",
  "hasVariant": [
    {"color": "Red", "size": "Medium", "price": "$899.99"},
    {"color": "Blue", "size": "Large", "price": "$899.99"},  
    {"color": "Green", "size": "Small", "price": "$849.99"}
  ]
}
```

## Security Test Cases

### URL Validation Tests
- ✅ Rejects non-HTTP/HTTPS schemes (ftp, file, javascript, data)
- ✅ Blocks localhost and 127.0.0.1
- ✅ Blocks private IP ranges (192.168.x, 10.x, 172.16-31.x)
- ✅ Blocks link-local addresses (169.254.x)
- ✅ Allows valid public HTTP/HTTPS URLs

### Input Sanitization Tests
- ✅ HTML sanitization removes `<script>` tags
- ✅ HTML sanitization removes `<style>` blocks  
- ✅ HTML sanitization removes HTML comments
- ✅ JSON-LD parsing handles malformed JSON gracefully
- ✅ Empty input handling

## Expected Test Results

When running the full test suite, you should see:

```
🧪 HTMLens Test Suite
====================
🔍 Running: Core library (default features)
✅ PASSED: Core library (default features)

🔍 Running: Core library (full-expansion)  
✅ PASSED: Core library (full-expansion)

🔍 Running: CLI unit tests
✅ PASSED: CLI unit tests

🔍 Running: CLI integration tests
✅ PASSED: CLI integration tests

📊 TEST SUMMARY
===============
Total tests: 15+
Passed: 15+
Failed: 0

🎉 All tests passed!
```

## Troubleshooting Tests

### Common Issues

1. **Compilation Errors**: Ensure all dependencies are installed with `cargo build --workspace`

2. **Worker Tests Fail**: Worker requires WASM target - use `cargo check --package htmlens-worker` instead

3. **Integration Tests Timeout**: CLI integration tests run actual processes - may be slow on first run

4. **Missing Test Fixtures**: Ensure `tests/fixtures/` directory exists with HTML/JSON files

### Debug Mode
```bash
# Run tests with debug output
RUST_LOG=debug cargo test --package htmlens-core -- --nocapture

# Run specific test
cargo test --package htmlens-core test_extract_json_ld_from_html -- --nocapture
```

## Continuous Integration

For CI/CD pipelines, use:

```bash
# Fast check (no integration tests)
cargo check --workspace
cargo test --workspace --lib

# Full test suite (longer)
./test_runner.sh
```

The test suite ensures HTMLens maintains quality across:
- Core parsing functionality 
- CLI user experience
- Worker security and API reliability
- Cross-platform compatibility