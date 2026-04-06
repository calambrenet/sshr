pub mod color;

// SPDX-License-Identifier: MIT OR Apache-2.0

/// Returns the user's home directory, or `None` if it cannot be determined.
///
/// Uses `HOME` (Unix) with fallback to `USERPROFILE` (Windows).
fn home_dir() -> Option<String> {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .ok()
}

/// Expands `~` to the user's home directory.
///
/// Only supports `~/...` and exact `~`. `~user/` is left unexpanded
/// (would require querying `/etc/passwd` or similar).
pub fn expand_tilde(path: &str) -> String {
    if path == "~" {
        return home_dir().unwrap_or_else(|| "~".to_string());
    }
    if let Some(rest) = path.strip_prefix("~/")
        && let Some(home) = home_dir()
    {
        return format!("{home}/{rest}");
    }
    path.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_tilde_with_path() {
        let home = home_dir().unwrap();
        assert_eq!(expand_tilde("~/foo"), format!("{home}/foo"));
    }

    #[test]
    fn test_expand_tilde_absolute_path() {
        assert_eq!(expand_tilde("/absolute/path"), "/absolute/path");
    }

    #[test]
    fn test_expand_tilde_alone() {
        let home = home_dir().unwrap();
        assert_eq!(expand_tilde("~"), home);
    }

    #[test]
    fn test_expand_tilde_user_unchanged() {
        assert_eq!(expand_tilde("~user/foo"), "~user/foo");
    }

    #[test]
    fn test_expand_tilde_empty() {
        assert_eq!(expand_tilde(""), "");
    }
}
