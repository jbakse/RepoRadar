use std::path::Path;
use std::process::Command;

use crate::models::*;

/// Default patterns that indicate high-risk (secrets/credentials)
const HIGH_RISK_PATTERNS: &[&str] = &[
    ".env",
    "*.key",
    "*.pem",
    "*.p12",
    "*.pfx",
    "id_rsa",
    "id_rsa.*",
    "id_ed25519",
    "id_ed25519.*",
    "*.mobileprovision",
    "credentials.json",
    "service-account*.json",
    "*.keystore",
    "*.jks",
];

/// Default patterns that indicate medium-risk (local state/data)
const MEDIUM_RISK_PATTERNS: &[&str] = &[
    "*.sqlite",
    "*.sqlite3",
    "*.db",
    ".env.local",
    ".env.development.local",
    ".env.production.local",
    "*.log",
    "local.settings.json",
];

/// Default patterns to always ignore (rebuildable artifacts)
const ALWAYS_IGNORE_PATTERNS: &[&str] = &[
    "node_modules",
    "dist",
    ".DS_Store",
    ".next",
    "__pycache__",
    "*.pyc",
    "target",
    ".venv",
    "venv",
    ".tox",
    "build",
    "*.o",
    "*.so",
    "*.dylib",
    ".sass-cache",
    "coverage",
    ".nyc_output",
    "*.class",
];

/// Find ignored files that exist on disk and classify their risk.
pub fn find_ignored_files(
    repo_path: &Path,
    flag_patterns: &[String],
    ignore_patterns: &[String],
) -> Vec<IgnoredFileInfo> {
    let mut results = Vec::new();

    // Get list of all files on disk (including ignored ones)
    // We use git ls-files to get ignored files
    let output = Command::new("git")
        .args(["-C", &repo_path.to_string_lossy()])
        .args(["ls-files", "--others", "--ignored", "--exclude-standard", "-z"])
        .output();

    let files = match output {
        Ok(out) if out.status.success() => {
            String::from_utf8_lossy(&out.stdout).to_string()
        }
        _ => return results,
    };

    let file_list: Vec<&str> = files.split('\0').filter(|f| !f.is_empty()).collect();
    if file_list.is_empty() {
        return results;
    }

    // Check each file against risk patterns
    for file_path in &file_list {
        let file_name = Path::new(file_path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        // Check if this should be ignored (rebuildable artifacts)
        if should_always_ignore(file_path, ignore_patterns) {
            continue;
        }

        // Classify risk level
        let (risk_level, category, matched_rule) =
            classify_risk(file_path, &file_name, flag_patterns);

        // Only include files that have some risk assessment
        if risk_level != RiskLevel::Low || matches_any_flag_pattern(&file_name, file_path, flag_patterns)
        {
            results.push(IgnoredFileInfo {
                path: file_path.to_string(),
                risk_level,
                matched_rule,
                category,
            });
        }
    }

    // Sort by risk level (High first)
    results.sort_by(|a, b| {
        let order = |r: &RiskLevel| match r {
            RiskLevel::High => 0,
            RiskLevel::Medium => 1,
            RiskLevel::Low => 2,
        };
        order(&a.risk_level).cmp(&order(&b.risk_level))
    });

    results
}

fn should_always_ignore(path: &str, ignore_patterns: &[String]) -> bool {
    let file_name = Path::new(path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    // Check against default always-ignore patterns
    for pattern in ALWAYS_IGNORE_PATTERNS {
        if matches_glob_simple(path, pattern) || matches_glob_simple(&file_name, pattern) {
            return true;
        }
    }

    // Check against user-configured ignore patterns
    for pattern in ignore_patterns {
        if matches_glob_simple(path, pattern) || matches_glob_simple(&file_name, pattern) {
            return true;
        }
    }

    // Check if path contains an always-ignore directory
    let path_components: Vec<&str> = path.split('/').collect();
    for component in &path_components {
        for pattern in ALWAYS_IGNORE_PATTERNS {
            if *component == *pattern {
                return true;
            }
        }
        for pattern in ignore_patterns {
            if *component == pattern.trim_end_matches("/**") {
                return true;
            }
        }
    }

    false
}

fn classify_risk(
    path: &str,
    file_name: &str,
    flag_patterns: &[String],
) -> (RiskLevel, String, String) {
    // Check high risk patterns (secrets/credentials)
    for pattern in HIGH_RISK_PATTERNS {
        if matches_glob_simple(file_name, pattern) || matches_glob_simple(path, pattern) {
            return (
                RiskLevel::High,
                "Secrets/Credentials".to_string(),
                pattern.to_string(),
            );
        }
    }

    // Check user-configured always-flag patterns
    for pattern in flag_patterns {
        if matches_glob_simple(file_name, pattern) || matches_glob_simple(path, pattern) {
            return (
                RiskLevel::High,
                "User-flagged important file".to_string(),
                pattern.to_string(),
            );
        }
    }

    // Check medium risk patterns
    for pattern in MEDIUM_RISK_PATTERNS {
        if matches_glob_simple(file_name, pattern) || matches_glob_simple(path, pattern) {
            return (
                RiskLevel::Medium,
                "Local data/state".to_string(),
                pattern.to_string(),
            );
        }
    }

    // Default: low risk
    (RiskLevel::Low, "Other ignored file".to_string(), String::new())
}

fn matches_any_flag_pattern(file_name: &str, path: &str, flag_patterns: &[String]) -> bool {
    for pattern in HIGH_RISK_PATTERNS {
        if matches_glob_simple(file_name, pattern) || matches_glob_simple(path, pattern) {
            return true;
        }
    }
    for pattern in flag_patterns {
        if matches_glob_simple(file_name, pattern) || matches_glob_simple(path, pattern) {
            return true;
        }
    }
    for pattern in MEDIUM_RISK_PATTERNS {
        if matches_glob_simple(file_name, pattern) || matches_glob_simple(path, pattern) {
            return true;
        }
    }
    false
}

/// Simple glob matching supporting * and ? wildcards
fn matches_glob_simple(text: &str, pattern: &str) -> bool {
    if pattern.is_empty() {
        return text.is_empty();
    }

    // Handle ** (match any path)
    if pattern.contains("**") {
        let parts: Vec<&str> = pattern.splitn(2, "**").collect();
        if parts.len() == 2 {
            let prefix = parts[0].trim_end_matches('/');
            let suffix = parts[1].trim_start_matches('/');

            if !prefix.is_empty() && !text.starts_with(prefix) {
                return false;
            }
            if !suffix.is_empty() && !text.ends_with(suffix) {
                return false;
            }
            return prefix.is_empty() && suffix.is_empty()
                || text.starts_with(prefix)
                || text.ends_with(suffix);
        }
    }

    // Simple glob with * and .
    let pattern_chars: Vec<char> = pattern.chars().collect();
    let text_chars: Vec<char> = text.chars().collect();
    glob_match_recursive(&text_chars, 0, &pattern_chars, 0)
}

fn glob_match_recursive(
    text: &[char],
    ti: usize,
    pattern: &[char],
    pi: usize,
) -> bool {
    if pi == pattern.len() {
        return ti == text.len();
    }

    if pattern[pi] == '*' {
        // Try matching zero or more characters
        for i in ti..=text.len() {
            if glob_match_recursive(text, i, pattern, pi + 1) {
                return true;
            }
        }
        return false;
    }

    if ti == text.len() {
        return false;
    }

    if pattern[pi] == '?' || pattern[pi] == text[ti] {
        return glob_match_recursive(text, ti + 1, pattern, pi + 1);
    }

    false
}

pub fn has_high_risk_files(
    repo_path: &Path,
    flag_patterns: &[String],
    ignore_patterns: &[String],
) -> bool {
    let files = find_ignored_files(repo_path, flag_patterns, ignore_patterns);
    files.iter().any(|f| f.risk_level == RiskLevel::High)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glob_matching() {
        assert!(matches_glob_simple(".env", ".env"));
        assert!(matches_glob_simple(".env.local", ".env.*"));
        assert!(matches_glob_simple("secret.key", "*.key"));
        assert!(matches_glob_simple("id_rsa", "id_rsa*"));
        assert!(matches_glob_simple("id_rsa.pub", "id_rsa*"));
        assert!(!matches_glob_simple("other.txt", "*.key"));
    }

    #[test]
    fn test_risk_classification() {
        let flag_patterns = vec![];
        let (risk, _, _) = classify_risk(".env", ".env", &flag_patterns);
        assert_eq!(risk, RiskLevel::High);

        let (risk, _, _) = classify_risk("data.sqlite", "data.sqlite", &flag_patterns);
        assert_eq!(risk, RiskLevel::Medium);

        let (risk, _, _) = classify_risk("readme.txt", "readme.txt", &flag_patterns);
        assert_eq!(risk, RiskLevel::Low);
    }
}
