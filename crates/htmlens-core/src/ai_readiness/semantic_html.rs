//! Semantic HTML and ARIA validation
//!
//! Validates that HTML uses proper semantic elements and ARIA attributes
//! for better AI understanding and accessibility. This helps AI-enabled browsers
//! parse and understand page structure.

use std::collections::{HashMap, HashSet};

use scraper::{ElementRef, Html, Selector};
use serde::{Deserialize, Serialize};

/// Results from semantic HTML analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticHtmlAnalysis {
    /// Landmark regions found
    pub landmarks: LandmarkAnalysis,

    /// Heading structure
    pub headings: HeadingAnalysis,

    /// Section and article heading usage
    pub sections: SectionAnalysis,

    /// Figure and caption usage
    pub figures: FigureAnalysis,

    /// Link text and accessible names
    pub links: LinkAnalysis,

    /// Generic versus semantic element usage
    pub element_usage: ElementUsageAnalysis,

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

    /// Number of visible <main> or role="main" landmarks
    pub main_count: usize,

    /// Has exactly one visible main landmark
    pub has_single_main: bool,

    /// Main landmarks nested in elements that cannot contain them
    pub invalid_main_nesting_count: usize,

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

/// Heading coverage for native sectioning elements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionAnalysis {
    /// <section> elements without their own heading
    pub sections_without_headings: usize,

    /// <article> elements without their own heading
    pub articles_without_headings: usize,
}

/// Figure and caption analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FigureAnalysis {
    /// Total <figure> elements
    pub total_figures: usize,

    /// Figures with a direct <figcaption> child
    pub figures_with_captions: usize,

    /// Figures without a direct <figcaption> child
    pub figures_without_captions: usize,
}

/// Link text and accessible-name analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkAnalysis {
    /// Total anchors with href attributes
    pub total_links: usize,

    /// Links without text or another accessible name
    pub links_without_names: usize,

    /// Links whose accessible name is generic without surrounding context
    pub generic_text_links: usize,
}

/// Generic versus semantic structural element usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementUsageAnalysis {
    /// Total <div> elements
    pub div_count: usize,

    /// Total native semantic structural elements
    pub semantic_container_count: usize,
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
    let sections = analyze_sections(&document);
    let figures = analyze_figures(&document);
    let links = analyze_links(&document);
    let element_usage = analyze_element_usage(&document);
    let aria = analyze_aria(&document);
    let forms = analyze_forms(&document);
    let images = analyze_images(&document);

    let mut issues = Vec::new();
    let mut recommendations = Vec::new();

    // Check for critical issues
    if landmarks.main_count == 0 {
        issues.push(
            "Missing main landmark - AI browsers use this to identify primary content".to_string(),
        );
        recommendations.push("Add a main element around your primary content".to_string());
    } else if !landmarks.has_single_main {
        issues.push(format!(
            "Found {} main landmarks - pages should expose exactly one primary content region",
            landmarks.main_count
        ));
        recommendations.push(
            "Keep one visible main element and hide or remove duplicate main landmarks".to_string(),
        );
    }

    if landmarks.invalid_main_nesting_count > 0 {
        issues.push(format!(
            "{} main landmark(s) are nested inside header, footer, nav, article, or aside elements",
            landmarks.invalid_main_nesting_count
        ));
        recommendations.push(
            "Move the main element outside header, footer, nav, article, and aside elements"
                .to_string(),
        );
    }

    if !headings.has_single_h1 {
        issues.push("Should have exactly one h1 per page for clear document structure".to_string());
        recommendations.push("Use a single h1 for the main page title".to_string());
    }

    if !headings.proper_hierarchy {
        issues.push(
            "Heading hierarchy has gaps - AI may misunderstand content structure".to_string(),
        );
        recommendations
            .push("Use heading levels in document order without upward skips".to_string());
    }

    if sections.sections_without_headings > 0 {
        issues.push(format!(
            "{} section element(s) do not have their own heading",
            sections.sections_without_headings
        ));
        recommendations.push("Add a descriptive heading to each section element".to_string());
    }

    if sections.articles_without_headings > 0 {
        issues.push(format!(
            "{} article element(s) do not have their own heading",
            sections.articles_without_headings
        ));
        recommendations.push("Add a descriptive heading to each article element".to_string());
    }

    if figures.figures_without_captions > 0 {
        recommendations.push(format!(
            "Add figcaption elements when the {} uncaptioned figure(s) need visible context",
            figures.figures_without_captions
        ));
    }

    if links.links_without_names > 0 {
        issues.push(format!(
            "{} link(s) have no text or accessible name",
            links.links_without_names
        ));
        recommendations.push("Give every link descriptive text or an accessible name".to_string());
    }

    if links.generic_text_links > 0 {
        recommendations.push(format!(
            "Replace generic text on {} link(s), such as 'click here' or 'read more', with destination-specific wording",
            links.generic_text_links
        ));
    }

    if element_usage.div_count >= 10 && element_usage.semantic_container_count == 0 {
        recommendations.push(format!(
            "Review the {} div elements and replace layout containers with semantic elements where they describe content structure",
            element_usage.div_count
        ));
    }

    if !aria.misuse_warnings.is_empty() {
        issues.extend(aria.misuse_warnings.iter().cloned());
        recommendations.push(
            "Remove redundant ARIA roles when a native HTML element already provides the same semantics"
                .to_string(),
        );
    }

    if forms.total_inputs > 0 {
        let label_percentage =
            (forms.labeled_inputs as f32 / forms.total_inputs as f32 * 100.0) as u32;
        if label_percentage < 90 {
            issues.push(format!(
                "Only {}% of form inputs have labels - AI needs labels to understand form purpose",
                label_percentage
            ));
            recommendations.push("Add label elements or aria-label to all form inputs".to_string());
        }
    }

    if images.total_images > 0 {
        let alt_percentage =
            (images.images_with_alt as f32 / images.total_images as f32 * 100.0) as u32;
        if alt_percentage < 90 {
            issues.push(format!(
                "{}% of images missing alt text - AI cannot understand image content",
                (100 - alt_percentage)
            ));
            recommendations.push("Add descriptive alt text to all meaningful images".to_string());
        }
    }

    SemanticHtmlAnalysis {
        landmarks,
        headings,
        sections,
        figures,
        links,
        element_usage,
        aria,
        forms,
        images,
        issues,
        recommendations,
    }
}

fn analyze_landmarks(document: &Html) -> LandmarkAnalysis {
    let main_count = count_semantic_or_role(document, "main", "main");
    let has_main = main_count > 0;
    let invalid_main_nesting_count = Selector::parse("main, [role]")
        .ok()
        .map(|selector| {
            document
                .select(&selector)
                .filter(|element| {
                    is_exposed(*element)
                        && (element.value().name() == "main" || has_role(*element, "main"))
                        && has_forbidden_main_ancestor(*element)
                })
                .count()
        })
        .unwrap_or(0);
    let has_navigation = count_semantic_or_role(document, "nav", "navigation") > 0;
    let has_header = count_semantic_or_role(document, "header", "banner") > 0;
    let has_footer = count_semantic_or_role(document, "footer", "contentinfo") > 0;

    let article_count = count_semantic_or_role(document, "article", "article");
    let section_count = count_exposed_elements(document, "section");
    let aside_count = count_semantic_or_role(document, "aside", "complementary");

    LandmarkAnalysis {
        has_main,
        main_count,
        has_single_main: main_count == 1,
        invalid_main_nesting_count,
        has_navigation,
        has_header,
        has_footer,
        article_count,
        section_count,
        aside_count,
    }
}

fn analyze_headings(document: &Html) -> HeadingAnalysis {
    let mut distribution = vec![0; 6];
    let mut proper_hierarchy = true;
    let mut hierarchy_issues = Vec::new();
    let mut previous_level = None;

    if let Ok(selector) = Selector::parse("h1, h2, h3, h4, h5, h6") {
        for heading in document
            .select(&selector)
            .filter(|heading| is_exposed(*heading))
        {
            let Some(current_level) = heading_level(heading) else {
                continue;
            };
            distribution[current_level - 1] += 1;

            if previous_level.is_none() && current_level > 1 {
                proper_hierarchy = false;
                push_unique(
                    &mut hierarchy_issues,
                    format!(
                        "Heading outline starts with <h{}> - start the page outline with <h1>",
                        current_level
                    ),
                );
            }

            if let Some(previous_level) = previous_level
                && current_level > previous_level + 1
            {
                proper_hierarchy = false;
                push_unique(
                    &mut hierarchy_issues,
                    format!(
                        "Heading hierarchy jumps from <h{}> to <h{}> - should be sequential",
                        previous_level, current_level
                    ),
                );
            }

            previous_level = Some(current_level);
        }
    }

    let has_single_h1 = distribution[0] == 1;

    HeadingAnalysis {
        has_single_h1,
        distribution,
        proper_hierarchy,
        hierarchy_issues,
    }
}

fn analyze_sections(document: &Html) -> SectionAnalysis {
    let mut sections_without_headings = 0;
    let mut articles_without_headings = 0;
    let Ok(sectioning_selector) = Selector::parse("section, article, [role]") else {
        return SectionAnalysis {
            sections_without_headings,
            articles_without_headings,
        };
    };
    let Ok(heading_selector) = Selector::parse("h1, h2, h3, h4, h5, h6") else {
        return SectionAnalysis {
            sections_without_headings,
            articles_without_headings,
        };
    };

    for sectioning_element in document
        .select(&sectioning_selector)
        .filter(|element| is_exposed(*element))
    {
        let has_own_heading = sectioning_element
            .select(&heading_selector)
            .any(|heading| is_exposed(heading) && heading_belongs_to(heading, sectioning_element));

        if !has_own_heading && sectioning_element.value().name() == "section" {
            sections_without_headings += 1;
        } else if !has_own_heading && is_article_element(sectioning_element) {
            articles_without_headings += 1;
        }
    }

    SectionAnalysis {
        sections_without_headings,
        articles_without_headings,
    }
}

fn analyze_figures(document: &Html) -> FigureAnalysis {
    let total_figures = count_exposed_elements(document, "figure");
    let figures_with_captions = Selector::parse("figure")
        .ok()
        .and_then(|figure_selector| {
            Selector::parse(":scope > figcaption")
                .ok()
                .map(|caption_selector| {
                    document
                        .select(&figure_selector)
                        .filter(|figure| {
                            is_exposed(*figure) && figure.select(&caption_selector).any(is_exposed)
                        })
                        .count()
                })
        })
        .unwrap_or(0);

    FigureAnalysis {
        total_figures,
        figures_with_captions,
        figures_without_captions: total_figures.saturating_sub(figures_with_captions),
    }
}

fn analyze_links(document: &Html) -> LinkAnalysis {
    let Ok(link_selector) = Selector::parse("a[href]") else {
        return LinkAnalysis {
            total_links: 0,
            links_without_names: 0,
            generic_text_links: 0,
        };
    };

    let mut total_links = 0;
    let mut links_without_names = 0;
    let mut generic_text_links = 0;
    let referenced_text = collect_referenced_text(document);

    for link in document
        .select(&link_selector)
        .filter(|link| is_exposed(*link))
    {
        total_links += 1;
        match link_accessible_name(link, &referenced_text) {
            Some(name) if is_generic_link_text(&name) => generic_text_links += 1,
            Some(_) => {}
            None => links_without_names += 1,
        }
    }

    LinkAnalysis {
        total_links,
        links_without_names,
        generic_text_links,
    }
}

fn analyze_element_usage(document: &Html) -> ElementUsageAnalysis {
    ElementUsageAnalysis {
        div_count: count_exposed_elements(document, "div"),
        semantic_container_count: count_exposed_elements(
            document,
            "header, nav, main, section, article, aside, footer, figure",
        ),
    }
}

fn analyze_aria(document: &Html) -> AriaAnalysis {
    let labeled_elements = count_exposed_elements(document, "[aria-label], [aria-labelledby]");
    let described_elements = count_exposed_elements(document, "[aria-describedby]");
    let live_regions = count_exposed_elements(document, "[aria-live]");
    let interactive_roles = Selector::parse("[role]")
        .ok()
        .map(|selector| {
            document
                .select(&selector)
                .filter(|element| {
                    is_exposed(*element)
                        && ["button", "link", "tab", "menuitem"]
                            .iter()
                            .any(|role| has_role(*element, role))
                })
                .count()
        })
        .unwrap_or(0);

    let mut misuse_warnings = Vec::new();

    // Check for common ARIA misuse
    if count_redundant_role(document, "button", "button") > 0 {
        misuse_warnings.push(
            "Found <button> with role='button' - redundant, native elements have implicit roles"
                .to_string(),
        );
    }

    if count_redundant_role(document, "a", "link") > 0 {
        misuse_warnings
            .push("Found <a> with role='link' - redundant, use semantic HTML instead".to_string());
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
    let form_count = count_exposed_elements(document, "form");
    let referenced_text = collect_referenced_text(document);
    let labeled_control_ids: HashSet<&str> = Selector::parse("label[for]")
        .ok()
        .map(|selector| {
            document
                .select(&selector)
                .filter(|label| !element_text_alternative(*label).is_empty())
                .filter_map(|label| non_empty_attr(label, "for"))
                .collect()
        })
        .unwrap_or_default();

    let (total_inputs, labeled_inputs) = Selector::parse("input, select, textarea")
        .ok()
        .map(|selector| {
            document
                .select(&selector)
                .filter(|control| is_exposed(*control) && !is_hidden_input(*control))
                .fold((0, 0), |(total, labeled), control| {
                    let is_labeled = referenced_name(control, &referenced_text).is_some()
                        || non_empty_attr(control, "aria-label").is_some()
                        || control
                            .attr("id")
                            .is_some_and(|id| labeled_control_ids.contains(id))
                        || named_ancestor_label(control)
                        || has_native_input_name(control);
                    (total + 1, labeled + usize::from(is_labeled))
                })
        })
        .unwrap_or((0, 0));

    let forms_with_fieldsets = Selector::parse("form")
        .ok()
        .and_then(|form_selector| {
            Selector::parse("fieldset").ok().map(|fieldset_selector| {
                document
                    .select(&form_selector)
                    .filter(|form| {
                        is_exposed(*form) && form.select(&fieldset_selector).any(is_exposed)
                    })
                    .count()
            })
        })
        .unwrap_or(0);
    let required_fields_marked =
        count_exposed_elements(document, "[required], [aria-required='true']") > 0;

    FormAnalysis {
        form_count,
        labeled_inputs,
        total_inputs,
        forms_with_fieldsets,
        required_fields_marked,
    }
}

fn analyze_images(document: &Html) -> ImageAnalysis {
    let total_images = count_exposed_elements(document, "img");
    let (images_with_alt, decorative_images) = Selector::parse("img")
        .ok()
        .map(|selector| {
            document
                .select(&selector)
                .filter(|image| is_exposed(*image))
                .fold((0, 0), |(with_alt, decorative), image| {
                    match image.attr("alt") {
                        Some(alt) if alt.trim().is_empty() => (with_alt + 1, decorative + 1),
                        Some(_) => (with_alt + 1, decorative),
                        None => (with_alt, decorative),
                    }
                })
        })
        .unwrap_or((0, 0));
    let images_missing_alt = total_images.saturating_sub(images_with_alt);

    ImageAnalysis {
        total_images,
        images_with_alt,
        decorative_images,
        images_missing_alt,
    }
}

fn has_forbidden_main_ancestor(element: ElementRef<'_>) -> bool {
    let mut ancestor = element.parent();
    while let Some(node) = ancestor {
        if let Some(element) = ElementRef::wrap(node)
            && (matches!(
                element.value().name(),
                "header" | "footer" | "nav" | "article" | "aside"
            ) || has_role(element, "banner")
                || has_role(element, "contentinfo")
                || has_role(element, "navigation")
                || has_role(element, "article")
                || has_role(element, "complementary"))
        {
            return true;
        }
        ancestor = node.parent();
    }
    false
}

fn is_exposed(element: ElementRef<'_>) -> bool {
    if hides_subtree(element) {
        return false;
    }

    let mut ancestor = element.parent();
    while let Some(node) = ancestor {
        if let Some(element) = ElementRef::wrap(node)
            && hides_subtree(element)
        {
            return false;
        }
        ancestor = node.parent();
    }
    true
}

fn hides_subtree(element: ElementRef<'_>) -> bool {
    element.attr("hidden").is_some()
        || element
            .attr("aria-hidden")
            .is_some_and(|value| value.eq_ignore_ascii_case("true"))
}

fn has_role(element: ElementRef<'_>, expected_role: &str) -> bool {
    element.attr("role").is_some_and(|roles| {
        roles
            .split_ascii_whitespace()
            .next()
            .is_some_and(|role| role.eq_ignore_ascii_case(expected_role))
    })
}

fn heading_level(heading: ElementRef<'_>) -> Option<usize> {
    heading.value().name().strip_prefix('h')?.parse().ok()
}

fn heading_belongs_to(heading: ElementRef<'_>, sectioning_element: ElementRef<'_>) -> bool {
    let mut ancestor = heading.parent();
    while let Some(node) = ancestor {
        if let Some(element) = ElementRef::wrap(node)
            && (element.value().name() == "section" || is_article_element(element))
        {
            return element == sectioning_element;
        }
        ancestor = node.parent();
    }
    false
}

fn is_article_element(element: ElementRef<'_>) -> bool {
    element.value().name() == "article" || has_role(element, "article")
}

fn is_hidden_input(element: ElementRef<'_>) -> bool {
    element.value().name() == "input"
        && element
            .attr("type")
            .is_some_and(|input_type| input_type.eq_ignore_ascii_case("hidden"))
}

fn has_native_input_name(element: ElementRef<'_>) -> bool {
    if element.value().name() != "input" {
        return false;
    }

    match element
        .attr("type")
        .unwrap_or("text")
        .to_ascii_lowercase()
        .as_str()
    {
        "submit" | "reset" => true,
        "button" => non_empty_attr(element, "value").is_some(),
        "image" => {
            non_empty_attr(element, "alt").is_some() || non_empty_attr(element, "title").is_some()
        }
        _ => false,
    }
}

fn named_ancestor_label(element: ElementRef<'_>) -> bool {
    let mut ancestor = element.parent();
    while let Some(node) = ancestor {
        if let Some(element) = ElementRef::wrap(node)
            && element.value().name() == "label"
        {
            return !element_text_alternative(element).is_empty();
        }
        ancestor = node.parent();
    }
    false
}

fn non_empty_attr<'a>(element: ElementRef<'a>, name: &str) -> Option<&'a str> {
    element.attr(name).filter(|value| !value.trim().is_empty())
}

fn link_accessible_name(
    link: ElementRef<'_>,
    referenced_text: &HashMap<String, String>,
) -> Option<String> {
    if let Some(name) = referenced_name(link, referenced_text) {
        return Some(name);
    }

    if let Some(label) = non_empty_attr(link, "aria-label") {
        return Some(normalize_text(label));
    }

    let text = element_text_alternative(link);
    if !text.is_empty() {
        return Some(text);
    }

    non_empty_attr(link, "title").map(normalize_text)
}

fn collect_referenced_text(document: &Html) -> HashMap<String, String> {
    let mut referenced_text = HashMap::new();
    let Ok(selector) = Selector::parse("[id]") else {
        return referenced_text;
    };

    for element in document.select(&selector) {
        let Some(id) = non_empty_attr(element, "id") else {
            continue;
        };
        let text = referenced_text_alternative(element);
        if !text.is_empty() {
            referenced_text.entry(id.to_string()).or_insert(text);
        }
    }

    referenced_text
}

fn referenced_name(
    element: ElementRef<'_>,
    referenced_text: &HashMap<String, String>,
) -> Option<String> {
    let references = non_empty_attr(element, "aria-labelledby")?;
    let mut seen = HashSet::new();
    let name = references
        .split_ascii_whitespace()
        .filter(|id| seen.insert(*id))
        .filter_map(|id| referenced_text.get(id))
        .cloned()
        .collect::<Vec<_>>()
        .join(" ");

    (!name.is_empty()).then_some(name)
}

fn element_text_alternative(element: ElementRef<'_>) -> String {
    if !is_exposed(element) {
        return String::new();
    }

    text_alternative(element, false)
}

fn referenced_text_alternative(element: ElementRef<'_>) -> String {
    text_alternative(element, true)
}

fn text_alternative(element: ElementRef<'_>, include_hidden_descendants: bool) -> String {
    let text = if include_hidden_descendants {
        normalize_text(&element.text().collect::<Vec<_>>().join(" "))
    } else {
        visible_descendant_text(element)
    };
    if !text.is_empty() {
        return text;
    }

    if let Some(value) = non_empty_attr(element, "value") {
        return normalize_text(value);
    }

    if let Ok(image_selector) = Selector::parse("img[alt]")
        && let Some(alt) = element
            .select(&image_selector)
            .filter(|image| include_hidden_descendants || is_exposed(*image))
            .find_map(|image| non_empty_attr(image, "alt"))
    {
        return normalize_text(alt);
    }

    String::new()
}

fn visible_descendant_text(element: ElementRef<'_>) -> String {
    let mut parts = Vec::new();
    collect_visible_descendant_text(element, &mut parts);
    normalize_text(&parts.join(" "))
}

fn collect_visible_descendant_text(element: ElementRef<'_>, parts: &mut Vec<String>) {
    for child in element.children() {
        if let Some(child_element) = ElementRef::wrap(child) {
            if !hides_subtree(child_element) {
                collect_visible_descendant_text(child_element, parts);
            }
        } else if let Some(text) = child.value().as_text() {
            parts.push(text.to_string());
        }
    }
}

fn normalize_text(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn is_generic_link_text(value: &str) -> bool {
    let normalized = value
        .trim()
        .trim_matches(|character: char| character.is_ascii_punctuation())
        .to_ascii_lowercase();
    matches!(
        normalized.as_str(),
        "click here"
            | "here"
            | "read more"
            | "learn more"
            | "more"
            | "details"
            | "continue"
            | "this"
            | "link"
    )
}

fn push_unique(values: &mut Vec<String>, value: String) {
    if !values.contains(&value) {
        values.push(value);
    }
}

fn count_semantic_or_role(document: &Html, element_name: &str, role: &str) -> usize {
    let selector_string = format!("{}, [role]", element_name);
    let Ok(selector) = Selector::parse(&selector_string) else {
        return 0;
    };

    document
        .select(&selector)
        .filter(|element| {
            is_exposed(*element)
                && (element.value().name() == element_name || has_role(*element, role))
        })
        .count()
}

fn count_redundant_role(document: &Html, element_name: &str, role: &str) -> usize {
    let Ok(selector) = Selector::parse(element_name) else {
        return 0;
    };

    document
        .select(&selector)
        .filter(|element| is_exposed(*element) && has_role(*element, role))
        .count()
}

fn count_exposed_elements(document: &Html, selector_str: &str) -> usize {
    let Ok(selector) = Selector::parse(selector_str) else {
        return 0;
    };

    document
        .select(&selector)
        .filter(|element| is_exposed(*element))
        .count()
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
        assert!(analysis.landmarks.has_single_main);
        assert_eq!(analysis.landmarks.invalid_main_nesting_count, 0);
        assert!(analysis.landmarks.has_navigation);
        assert!(analysis.landmarks.has_header);
        assert!(analysis.headings.has_single_h1);
        assert!(analysis.headings.proper_hierarchy);
        assert_eq!(analysis.sections.articles_without_headings, 0);
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

    #[test]
    fn test_heading_hierarchy_uses_document_order() {
        let html = r#"
        <main>
            <h1>Main title</h1>
            <h3>Skipped level</h3>
            <h2>Later section</h2>
        </main>
        "#;

        let analysis = analyze_semantic_html(html);
        assert!(!analysis.headings.proper_hierarchy);
        assert!(
            analysis
                .headings
                .hierarchy_issues
                .iter()
                .any(|issue| issue.contains("<h1> to <h3>"))
        );
    }

    #[test]
    fn test_main_count_and_invalid_nesting() {
        let html = r#"
        <header><main><h1>Nested main</h1></main></header>
        <main><h2>Second main</h2></main>
        "#;

        let analysis = analyze_semantic_html(html);
        assert_eq!(analysis.landmarks.main_count, 2);
        assert!(!analysis.landmarks.has_single_main);
        assert_eq!(analysis.landmarks.invalid_main_nesting_count, 1);
        assert!(
            analysis
                .issues
                .iter()
                .any(|issue| issue.contains("2 main landmarks"))
        );
    }

    #[test]
    fn test_role_landmarks_count_as_forbidden_main_ancestors() {
        let html = r#"
        <div role="navigation">
            <div role="main"><h1>Nested main</h1></div>
        </div>
        "#;

        let analysis = analyze_semantic_html(html);
        assert_eq!(analysis.landmarks.main_count, 1);
        assert_eq!(analysis.landmarks.invalid_main_nesting_count, 1);
    }

    #[test]
    fn test_section_heading_belongs_to_nearest_sectioning_element() {
        let html = r#"
        <main>
            <h1>Main title</h1>
            <section>
                <article><h2>Article title</h2></article>
            </section>
        </main>
        "#;

        let analysis = analyze_semantic_html(html);
        assert_eq!(analysis.sections.sections_without_headings, 1);
        assert_eq!(analysis.sections.articles_without_headings, 0);
    }

    #[test]
    fn test_figure_and_link_analysis() {
        let html = r#"
        <main>
            <h1>Main title</h1>
            <figure><img src="chart.png" alt="Quarterly growth"><figcaption>Growth</figcaption></figure>
            <figure><img src="team.png" alt="Team"></figure>
            <a href="/report">Read the quarterly report</a>
            <a href="/about">Read more</a>
            <a href="/search" aria-label="Search"></a>
            <a href="/empty"></a>
        </main>
        "#;

        let analysis = analyze_semantic_html(html);
        assert_eq!(analysis.figures.total_figures, 2);
        assert_eq!(analysis.figures.figures_with_captions, 1);
        assert_eq!(analysis.figures.figures_without_captions, 1);
        assert_eq!(analysis.links.total_links, 4);
        assert_eq!(analysis.links.generic_text_links, 1);
        assert_eq!(analysis.links.links_without_names, 1);
    }

    #[test]
    fn test_form_labels_are_counted_once_across_supported_patterns() {
        let html = r#"
        <main>
            <h1>Contact</h1>
            <form>
                <label for="email">Email</label><input id="email">
                <label>Country<select><option>NL</option></select></label>
                <span id="message-label">Message</span><textarea aria-labelledby="message-label"></textarea>
                <input aria-label="Search">
            </form>
        </main>
        "#;

        let analysis = analyze_semantic_html(html);
        assert_eq!(analysis.forms.total_inputs, 4);
        assert_eq!(analysis.forms.labeled_inputs, 4);
    }

    #[test]
    fn test_hidden_content_is_excluded_from_structural_counts() {
        let html = r#"
        <main><h1>Visible title</h1></main>
        <main hidden><h1>Hidden title</h1></main>
        <div aria-hidden="true">
            <nav>Hidden navigation</nav>
            <img src="hidden.png">
        </div>
        "#;

        let analysis = analyze_semantic_html(html);
        assert_eq!(analysis.landmarks.main_count, 1);
        assert_eq!(analysis.headings.distribution[0], 1);
        assert!(!analysis.landmarks.has_navigation);
        assert_eq!(analysis.images.total_images, 0);
    }

    #[test]
    fn test_role_matching_is_case_insensitive_and_token_aware() {
        let html = r#"
        <div role="MAIN document"><h1>Title</h1></div>
        <div role="Navigation region">Navigation</div>
        "#;

        let analysis = analyze_semantic_html(html);
        assert!(analysis.landmarks.has_single_main);
        assert!(analysis.landmarks.has_navigation);
    }

    #[test]
    fn test_accessible_name_references_must_resolve_to_text() {
        let html = r#"
        <main>
            <h1 id="title">Quarterly report</h1>
            <a id="report-link" href="/report" aria-labelledby="report-link title">Read more</a>
            <a href="/empty" aria-labelledby="missing"></a>
            <form>
                <input aria-labelledby="missing">
                <label for="empty"></label><input id="empty">
            </form>
        </main>
        "#;

        let analysis = analyze_semantic_html(html);
        assert_eq!(analysis.links.total_links, 2);
        assert_eq!(analysis.links.generic_text_links, 0);
        assert_eq!(analysis.links.links_without_names, 1);
        assert_eq!(analysis.forms.total_inputs, 2);
        assert_eq!(analysis.forms.labeled_inputs, 0);
    }

    #[test]
    fn test_hidden_descendants_do_not_create_native_accessible_names() {
        let html = r#"
        <main>
            <h1>Report</h1>
            <a href="/hidden"><span aria-hidden="true">Hidden link name</span></a>
            <a href="/hidden-image"><span hidden><img alt="Hidden image name"></span></a>
            <span id="referenced-name" hidden>Explicitly referenced name</span>
            <a href="/referenced" aria-labelledby="referenced-name"></a>
            <form>
                <label for="hidden-label"><span hidden>Hidden label</span></label>
                <input id="hidden-label">
                <input aria-labelledby="referenced-name">
            </form>
        </main>
        "#;

        let analysis = analyze_semantic_html(html);
        assert_eq!(analysis.links.total_links, 3);
        assert_eq!(analysis.links.links_without_names, 2);
        assert_eq!(analysis.forms.total_inputs, 2);
        assert_eq!(analysis.forms.labeled_inputs, 1);
    }

    #[test]
    fn test_role_articles_use_the_same_heading_ownership_rules() {
        let html = r#"
        <main>
            <h1>News</h1>
            <div role="article"><h2>Named update</h2></div>
            <div role="article"><p>Untitled update</p></div>
        </main>
        "#;

        let analysis = analyze_semantic_html(html);
        assert_eq!(analysis.landmarks.article_count, 2);
        assert_eq!(analysis.sections.articles_without_headings, 1);
    }

    #[test]
    fn test_button_style_inputs_use_native_accessible_names() {
        let html = r#"
        <main>
            <h1>Search</h1>
            <form>
                <input type="submit">
                <input type="reset">
                <input type="button" value="Open filters">
                <input type="image" src="search.png" alt="Search">
                <input type="button">
                <input type="HIDDEN">
            </form>
        </main>
        "#;

        let analysis = analyze_semantic_html(html);
        assert_eq!(analysis.forms.total_inputs, 5);
        assert_eq!(analysis.forms.labeled_inputs, 4);
    }
}
