//! AI Readiness checking functionality
//!
//! This module provides tools to check how well a website communicates
//! with AI agents through standard specifications and files.

pub mod mcp_manifest;
pub mod openapi;
pub mod plugin_manifest;
pub mod robots_txt;
pub mod semantic_html;
pub mod sitemap;
pub mod well_known;

use serde::{Deserialize, Serialize};

/// Overall AI readiness assessment for a website
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiReadinessReport {
    /// URL that was checked
    pub url: String,

    /// Overall readiness score (0-100)
    pub score: u8,

    /// Results from checking .well-known directory
    pub well_known: well_known::WellKnownChecks,

    /// Summary of what's working well
    pub strengths: Vec<String>,

    /// Issues that need attention
    pub issues: Vec<AiReadinessIssue>,

    /// Recommendations for improvement
    pub recommendations: Vec<String>,
}

/// An issue found during AI readiness checking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiReadinessIssue {
    /// Issue severity
    pub severity: IssueSeverity,

    /// Category of the issue
    pub category: String,

    /// Description of the issue
    pub message: String,

    /// Optional link to documentation
    pub reference: Option<String>,
}

/// Severity level for AI readiness issues
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IssueSeverity {
    /// Critical issue - blocks AI integration
    Critical,

    /// High priority - strongly recommended
    High,

    /// Medium priority - recommended
    Medium,

    /// Low priority - nice to have
    Low,
}

impl AiReadinessReport {
    /// Create a new AI readiness report
    pub fn new(url: String) -> Self {
        Self {
            url,
            score: 0,
            well_known: well_known::WellKnownChecks::default(),
            strengths: Vec::new(),
            issues: Vec::new(),
            recommendations: Vec::new(),
        }
    }

    /// Calculate the overall score based on checks
    pub fn calculate_score(&mut self) {
        let mut score = 100u32;

        // Deduct points based on issue severity
        for issue in &self.issues {
            let deduction = match issue.severity {
                IssueSeverity::Critical => 20,
                IssueSeverity::High => 10,
                IssueSeverity::Medium => 5,
                IssueSeverity::Low => 2,
            };
            score = score.saturating_sub(deduction);
        }

        self.score = score.min(100) as u8;
    }
}
