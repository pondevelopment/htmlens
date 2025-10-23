//! Semantic HTML and ARIA validation
//!
//! Validates that HTML uses proper semantic elements and ARIA attributes
//! for better AI understanding and accessibility. This helps AI-enabled browsers
//! parse and understand page structure.

use serde::{Deserialize, Serialize};
use scraper::{Html, Selector};

/// Results from semantic HTML analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticHtmlAnalysis {
    /// Landmark regions found
    pub landmarks: LandmarkAnalysis,
    
    /// Heading structure
    pub headings: HeadingAnalysis,
    
    /// ARIA usage
    pub aria: AriaAnalysis,
    
    /// Form accessibility
    pub forms: FormAnalysis,
    
    /// Image accessibility
    pub images: ImageAnalysis,
    
    /// Issues found
    pub issues: Vec<String>,
    
    /// Recommendations
    pub recommendations: Vec<String>,
}

/// Landmark region analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LandmarkAnalysis {
    /// Has <main> or role="main"
    pub has_main: bool,
    
    /// Has <nav> or role="navigation"
    pub has_navigation: bool,
    
    /// Has <header> or role="banner"
    pub has_header: bool,
    
    /// Has <footer> or role="contentinfo"
    pub has_footer: bool,
    
    /// Count of <article> or role="article"
    pub article_count: usize,
    
    /// Count of <section> elements
    pub section_count: usize,
    
    /// Count of <aside> or role="complementary"
    pub aside_count: usize,
}

/// Heading hierarchy analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeadingAnalysis {
    /// Has exactly one <h1>
    pub has_single_h1: bool,
    
    /// Heading distribution (h1, h2, h3, h4, h5, h6)
    pub distribution: Vec<usize>,
    
    /// Has proper hierarchy (no skipped levels)
    pub proper_hierarchy: bool,
    
    /// Issues with heading structure
    pub hierarchy_issues: Vec<String>,
}

/// ARIA attribute usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AriaAnalysis {
    /// Elements with aria-label
    pub labeled_elements: usize,
    
    /// Elements with aria-describedby
    pub described_elements: usize,
    
    /// Live regions (aria-live)
    pub live_regions: usize,
    
    /// Interactive elements with roles
    pub interactive_roles: usize,
    
    /// Potential ARIA misuse
    pub misuse_warnings: Vec<String>,
}

/// Form accessibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormAnalysis {
    /// Total forms found
    pub form_count: usize,
    
    /// Inputs with associated labels
    pub labeled_inputs: usize,
    
    /// Total inputs
    pub total_inputs: usize,
    
    /// Forms with fieldsets
    pub forms_with_fieldsets: usize,
    
    /// Required fields properly marked
    pub required_fields_marked: bool,
}

/// Image accessibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageAnalysis {
    /// Total images
    pub total_images: usize,
    
    /// Images with alt text
    pub images_with_alt: usize,
    
    /// Decorative images (alt="")
    pub decorative_images: usize,
    
    /// Images missing alt
    pub images_missing_alt: usize,
}

/// Analyze semantic HTML structure from HTML content
pub fn analyze_semantic_html(html: &str) -> SemanticHtmlAnalysis {
    let document = Html::parse_document(html);
    
    let landmarks = analyze_landmarks(&document);
    let headings = analyze_headings(&document);
    let aria = analyze_aria(&document);
    let forms = analyze_forms(&document);
    let images = analyze_images(&document);
    
    let mut issues = Vec::new();
    let mut recommendations = Vec::new();
    
    // Check for critical issues
    if !landmarks.has_main {
        issues.push("Missing main landmark - AI browsers use this to identify primary content".to_string());
        recommendations.push("Add a main element around your primary content".to_string());
    }
    
    if !headings.has_single_h1 {
        issues.push("Should have exactly one h1 per page for clear document structure".to_string());
        recommendations.push("Use a single h1 for the main page title".to_string());
    }
    
    if !headings.proper_hierarchy {
        issues.push("Heading hierarchy has gaps - AI may misunderstand content structure".to_string());
    }
    
    if forms.total_inputs > 0 {
        let label_percentage = (forms.labeled_inputs as f32 / forms.total_inputs as f32 * 100.0) as u32;
        if label_percentage < 80 {
            issues.push(format!("Only {}% of form inputs have labels - AI needs labels to understand form purpose", label_percentage));
            recommendations.push("Add label elements or aria-label to all form inputs".to_string());
        }
    }
    
    if images.total_images > 0 {
        let alt_percentage = (images.images_with_alt as f32 / images.total_images as f32 * 100.0) as u32;
        if alt_percentage < 90 {
            issues.push(format!("{}% of images missing alt text - AI cannot understand image content", 
                (100 - alt_percentage)));
            recommendations.push("Add descriptive alt text to all meaningful images".to_string());
        }
    }
    
    SemanticHtmlAnalysis {
        landmarks,
        headings,
        aria,
        forms,
        images,
        issues,
        recommendations,
    }
}

fn analyze_landmarks(document: &Html) -> LandmarkAnalysis {
    // Check for semantic landmarks
    let has_main = select_exists(document, "main") || select_exists(document, "[role='main']");
    let has_navigation = select_exists(document, "nav") || select_exists(document, "[role='navigation']");
    let has_header = select_exists(document, "header") || select_exists(document, "[role='banner']");
    let has_footer = select_exists(document, "footer") || select_exists(document, "[role='contentinfo']");
    
    let article_count = count_elements(document, "article") + count_elements(document, "[role='article']");
    let section_count = count_elements(document, "section");
    let aside_count = count_elements(document, "aside") + count_elements(document, "[role='complementary']");
    
    LandmarkAnalysis {
        has_main,
        has_navigation,
        has_header,
        has_footer,
        article_count,
        section_count,
        aside_count,
    }
}

fn analyze_headings(document: &Html) -> HeadingAnalysis {
    let h1_count = count_elements(document, "h1");
    let h2_count = count_elements(document, "h2");
    let h3_count = count_elements(document, "h3");
    let h4_count = count_elements(document, "h4");
    let h5_count = count_elements(document, "h5");
    let h6_count = count_elements(document, "h6");
    
    let distribution = vec![h1_count, h2_count, h3_count, h4_count, h5_count, h6_count];
    let has_single_h1 = h1_count == 1;
    
    // Check hierarchy (no skipped levels)
    let mut proper_hierarchy = true;
    let mut hierarchy_issues = Vec::new();
    let mut last_level = 0;
    
    for (level, &count) in distribution.iter().enumerate() {
        if count > 0 {
            let current_level = level + 1;
            if last_level > 0 && current_level > last_level + 1 {
                proper_hierarchy = false;
                hierarchy_issues.push(format!(
                    "Heading hierarchy jumps from <h{}> to <h{}> - should be sequential",
                    last_level, current_level
                ));
            }
            last_level = current_level;
        }
    }
    
    HeadingAnalysis {
        has_single_h1,
        distribution,
        proper_hierarchy,
        hierarchy_issues,
    }
}

fn analyze_aria(document: &Html) -> AriaAnalysis {
    let labeled_elements = count_elements(document, "[aria-label]");
    let described_elements = count_elements(document, "[aria-describedby]");
    let live_regions = count_elements(document, "[aria-live]");
    let interactive_roles = count_elements(document, "[role='button']") +
                           count_elements(document, "[role='link']") +
                           count_elements(document, "[role='tab']") +
                           count_elements(document, "[role='menuitem']");
    
    let mut misuse_warnings = Vec::new();
    
    // Check for common ARIA misuse
    if count_elements(document, "button[role='button']") > 0 {
        misuse_warnings.push("Found <button> with role='button' - redundant, native elements have implicit roles".to_string());
    }
    
    if count_elements(document, "a[role='link']") > 0 {
        misuse_warnings.push("Found <a> with role='link' - redundant, use semantic HTML instead".to_string());
    }
    
    AriaAnalysis {
        labeled_elements,
        described_elements,
        live_regions,
        interactive_roles,
        misuse_warnings,
    }
}

fn analyze_forms(document: &Html) -> FormAnalysis {
    let form_count = count_elements(document, "form");
    let total_inputs = count_elements(document, "input:not([type='hidden'])") +
                       count_elements(document, "select") +
                       count_elements(document, "textarea");
    
    // Count inputs with labels (either <label> or aria-label)
    let labeled_inputs = count_elements(document, "input[id] + label, label input") +
                        count_elements(document, "input[aria-label]") +
                        count_elements(document, "select[aria-label]") +
                        count_elements(document, "textarea[aria-label]");
    
    let forms_with_fieldsets = count_elements(document, "form fieldset");
    let required_fields_marked = count_elements(document, "[required], [aria-required='true']") > 0;
    
    FormAnalysis {
        form_count,
        labeled_inputs,
        total_inputs,
        forms_with_fieldsets,
        required_fields_marked,
    }
}

fn analyze_images(document: &Html) -> ImageAnalysis {
    let total_images = count_elements(document, "img");
    let images_with_alt = count_elements(document, "img[alt]");
    let decorative_images = count_elements(document, "img[alt='']");
    let images_missing_alt = total_images.saturating_sub(images_with_alt);
    
    ImageAnalysis {
        total_images,
        images_with_alt,
        decorative_images,
        images_missing_alt,
    }
}

// Helper functions
fn select_exists(document: &Html, selector_str: &str) -> bool {
    if let Ok(selector) = Selector::parse(selector_str) {
        document.select(&selector).next().is_some()
    } else {
        false
    }
}

fn count_elements(document: &Html, selector_str: &str) -> usize {
    if let Ok(selector) = Selector::parse(selector_str) {
        document.select(&selector).count()
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_good_semantic_html() {
        let html = r#"
        <!DOCTYPE html>
        <html>
        <head><title>Test</title></head>
        <body>
            <header><h1>Main Title</h1></header>
            <nav><a href="/">Home</a></nav>
            <main>
                <article>
                    <h2>Article Title</h2>
                    <p>Content</p>
                </article>
            </main>
            <footer>Footer</footer>
        </body>
        </html>
        "#;
        
        let analysis = analyze_semantic_html(html);
        assert!(analysis.landmarks.has_main);
        assert!(analysis.landmarks.has_navigation);
        assert!(analysis.landmarks.has_header);
        assert!(analysis.headings.has_single_h1);
    }
    
    #[test]
    fn test_poor_semantic_html() {
        let html = r#"
        <!DOCTYPE html>
        <html>
        <body>
            <div class="header"><div class="title">Title</div></div>
            <div class="content">Content</div>
        </body>
        </html>
        "#;
        
        let analysis = analyze_semantic_html(html);
        assert!(!analysis.landmarks.has_main);
        assert!(!analysis.headings.has_single_h1);
        assert!(!analysis.issues.is_empty());
    }
}
