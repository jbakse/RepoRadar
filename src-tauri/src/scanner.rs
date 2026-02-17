use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Scan root folders for Git repositories by detecting .git/ directories or .git files.
pub fn discover_repos(roots: &[String], exclusions: &[String]) -> Vec<PathBuf> {
    let mut repos = Vec::new();

    for root in roots {
        let root_path = Path::new(root);
        if !root_path.exists() || !root_path.is_dir() {
            continue;
        }

        let walker = WalkDir::new(root_path)
            .follow_links(false)
            .into_iter()
            .filter_entry(|entry| {
                let file_name = entry.file_name().to_string_lossy();
                if entry.depth() > 0 && entry.file_type().is_dir() {
                    // Never recurse into .git directories (handled separately below)
                    if file_name == ".git" {
                        return false;
                    }
                    // Don't recurse into user-configured excluded directories
                    for exclusion in exclusions {
                        if file_name == *exclusion {
                            return false;
                        }
                    }
                }
                true
            });

        for entry in walker.filter_map(|e| e.ok()) {
            if entry.file_type().is_dir() {
                // Check if this directory contains a .git entry (dir or file)
                let git_indicator = entry.path().join(".git");
                if git_indicator.exists() {
                    let repo_path = entry.path().to_path_buf();
                    if !repos.contains(&repo_path) {
                        repos.push(repo_path);
                    }
                }
            }
        }
    }

    repos.sort();
    repos
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_discover_repos_finds_git_dirs() {
        let tmp = std::env::temp_dir().join("reporadar_test_discover");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(tmp.join("repo1/.git")).unwrap();
        fs::create_dir_all(tmp.join("repo2/.git")).unwrap();
        fs::create_dir_all(tmp.join("not_a_repo")).unwrap();

        let roots = vec![tmp.to_string_lossy().to_string()];
        let exclusions = vec!["node_modules".to_string()];
        let repos = discover_repos(&roots, &exclusions);

        assert_eq!(repos.len(), 2);
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_excludes_node_modules() {
        let tmp = std::env::temp_dir().join("reporadar_test_exclude");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(tmp.join("project/.git")).unwrap();
        fs::create_dir_all(tmp.join("project/node_modules/dep/.git")).unwrap();

        let roots = vec![tmp.to_string_lossy().to_string()];
        let exclusions = vec!["node_modules".to_string()];
        let repos = discover_repos(&roots, &exclusions);

        assert_eq!(repos.len(), 1);
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_discovers_root_folder_as_repo() {
        let tmp = std::env::temp_dir().join("reporadar_test_root_repo");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(tmp.join(".git")).unwrap();

        let roots = vec![tmp.to_string_lossy().to_string()];
        let exclusions = vec!["node_modules".to_string()];
        let repos = discover_repos(&roots, &exclusions);

        assert_eq!(repos.len(), 1);
        assert_eq!(repos[0], tmp);
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_git_not_blocked_by_exclusions() {
        // Regression test: .git must never be user-excludable
        let tmp = std::env::temp_dir().join("reporadar_test_git_excl");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(tmp.join("project/.git")).unwrap();

        let roots = vec![tmp.to_string_lossy().to_string()];
        // Even if someone puts .git in exclusions, repos should still be found
        let exclusions = vec![".git".to_string()];
        let repos = discover_repos(&roots, &exclusions);

        assert_eq!(repos.len(), 1);
        let _ = fs::remove_dir_all(&tmp);
    }
}
