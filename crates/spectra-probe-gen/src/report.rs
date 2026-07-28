//! Rapport de conversion.

use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Serialize, Default)]
pub struct ConversionReport {
    pub sources: HashMap<String, SourceReport>,
    pub errors: Vec<ErrorEntry>,
}

#[derive(Debug, Serialize, Default)]
pub struct SourceReport {
    pub converted: usize,
    pub rejected: usize,
    pub rejected_details: Vec<RejectionEntry>,
}

#[derive(Debug, Serialize)]
pub struct RejectionEntry {
    pub name: String,
    pub reason: String,
}

#[derive(Debug, Serialize)]
pub struct ErrorEntry {
    pub source: String,
    pub error: String,
}

impl ConversionReport {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reject(&mut self, source: &str, name: String, reason: String) {
        let src = self.sources.entry(source.to_string()).or_default();
        src.rejected += 1;
        src.rejected_details.push(RejectionEntry { name, reason });
    }

    pub fn error(&mut self, source: &str, error: String) {
        self.errors.push(ErrorEntry {
            source: source.to_string(),
            error,
        });
    }

    pub fn merge(&mut self, other: ConversionReport) {
        for (source, report) in other.sources {
            let entry = self.sources.entry(source).or_default();
            entry.converted += report.converted;
            entry.rejected += report.rejected;
            entry.rejected_details.extend(report.rejected_details);
        }
        self.errors.extend(other.errors);
    }

    pub fn summary(&self) -> String {
        let total_converted: usize = self.sources.values().map(|s| s.converted).sum();
        let total_rejected: usize = self.sources.values().map(|s| s.rejected).sum();
        format!(
            "Total: {} sondes converties, {} rejetées, {} erreurs",
            total_converted, total_rejected, self.errors.len()
        )
    }
}
