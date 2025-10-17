//! Integration tests for htmlens CLI
//!
//! These tests run the CLI as a subprocess to test end-to-end functionality

use std::env;
use std::path::Path;
use std::process::Command;

fn get_cli_binary() -> String {
    // Try to find the binary in various locations
    let binary_name = if cfg!(target_os = "windows") {
        "htmlens.exe"
    } else {
        "htmlens"
    };

    // Check if we're in the target directory
    if let Ok(target_dir) = env::var("CARGO_TARGET_DIR") {
        let binary_path = Path::new(&target_dir).join("debug").join(binary_name);
        if binary_path.exists() {
            return binary_path.to_string_lossy().to_string();
        }
    }

    // Default to cargo run for tests
    binary_name.to_string()
}

#[test]
fn test_cli_help() {
    let output = Command::new("cargo")
        .args(&["run", "--", "--help"])
        .current_dir("../../..") // Go to workspace root
        .output()
        .expect("Failed to run CLI");

    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.contains("htmlens — A semantic lens for the web"));
    assert!(stdout.contains("Usage:"));
    assert!(stdout.contains("--help"));
    assert!(stdout.contains("Developed by Pon Datalab"));
}

#[test]
fn test_cli_version() {
    let output = Command::new("cargo")
        .args(&["run", "--", "--version"])
        .current_dir("../../..")
        .output()
        .expect("Failed to run CLI");

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("htmlens 0.4.0"));
}

#[test]
fn test_cli_direct_json_ld() {
    let json_ld = r#"{"@context": "https://schema.org", "@type": "Product", "name": "Test Product", "description": "A test product"}"#;

    let output = Command::new("cargo")
        .args(&["run", "--", json_ld])
        .current_dir("../../..")
        .output()
        .expect("Failed to run CLI");

    let stdout = String::from_utf8(output.stdout).unwrap();

    // Should process JSON-LD and generate output
    assert!(stdout.contains("Test Product"));
    assert!(stdout.len() > 100); // Should have substantial output
}

#[test]
fn test_cli_graph_only_mode() {
    let json_ld =
        r#"{"@context": "https://schema.org", "@type": "Product", "name": "Test Product"}"#;

    let output = Command::new("cargo")
        .args(&["run", "--", "-g", json_ld])
        .current_dir("../../..")
        .output()
        .expect("Failed to run CLI");

    let stdout = String::from_utf8(output.stdout).unwrap();

    // In graph-only mode, should be more concise
    assert!(stdout.contains("Product"));
    assert!(stdout.len() < 500); // Should be shorter than full output
}

#[test]
fn test_cli_invalid_json() {
    let invalid_json = r#"{"@type": "Product", "name": "Invalid""#; // Missing closing brace

    let output = Command::new("cargo")
        .args(&["run", "--", invalid_json])
        .current_dir("../../..")
        .output()
        .expect("Failed to run CLI");

    // Should handle invalid JSON gracefully
    assert!(!output.status.success() || !String::from_utf8(output.stderr).unwrap().is_empty());
}

#[test]
fn test_cli_invalid_url() {
    let output = Command::new("cargo")
        .args(&["run", "--", "not-a-url"])
        .current_dir("../../..")
        .output()
        .expect("Failed to run CLI");

    // Should handle invalid URLs gracefully
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("Error") || stderr.contains("Failed") || !output.status.success());
}

#[test]
fn test_cli_mermaid_flag() {
    let json_ld =
        r#"{"@context": "https://schema.org", "@type": "Product", "name": "Test Product"}"#;

    let output = Command::new("cargo")
        .args(&["run", "--", "-m", json_ld])
        .current_dir("../../..")
        .output()
        .expect("Failed to run CLI");

    let stdout = String::from_utf8(output.stdout).unwrap();

    // Should include mermaid diagram
    assert!(stdout.contains("mermaid") || stdout.contains("graph"));
}

#[test]
fn test_cli_complex_product_group() {
    let json_ld = r#"{
        "@context": "https://schema.org",
        "@type": "ProductGroup",
        "name": "Bike Collection",
        "hasVariant": [
            {
                "@type": "Product",
                "name": "Red Bike",
                "color": "Red",
                "offers": {
                    "@type": "Offer",
                    "price": "299.99",
                    "priceCurrency": "USD"
                }
            },
            {
                "@type": "Product", 
                "name": "Blue Bike",
                "color": "Blue",
                "offers": {
                    "@type": "Offer",
                    "price": "349.99",
                    "priceCurrency": "USD"
                }
            }
        ]
    }"#;

    let output = Command::new("cargo")
        .args(&["run", "--", json_ld])
        .current_dir("../../..")
        .output()
        .expect("Failed to run CLI");

    let stdout = String::from_utf8(output.stdout).unwrap();

    // Should show product variants and pricing
    assert!(stdout.contains("Bike Collection"));
    assert!(stdout.contains("Red Bike"));
    assert!(stdout.contains("Blue Bike"));
    assert!(stdout.contains("299.99"));
    assert!(stdout.contains("349.99"));
    assert!(stdout.contains("Color")); // Should show varying properties
}
