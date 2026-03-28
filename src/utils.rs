// SPDX-License-Identifier: MIT OR Apache-2.0

/// Expande `~` al directorio home del usuario.
///
/// Solo soporta `~/...` y `~` exacto. `~user/` se deja sin expandir
/// (requeriría consultar `/etc/passwd` o similar).
pub fn expand_tilde(path: &str) -> String {
    if path == "~" {
        return std::env::var("HOME").unwrap_or_else(|_| "~".to_string());
    }
    if let Some(rest) = path.strip_prefix("~/")
        && let Ok(home) = std::env::var("HOME")
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
        let home = std::env::var("HOME").unwrap();
        assert_eq!(expand_tilde("~/foo"), format!("{home}/foo"));
    }

    #[test]
    fn test_expand_tilde_absolute_path() {
        assert_eq!(expand_tilde("/absolute/path"), "/absolute/path");
    }

    #[test]
    fn test_expand_tilde_alone() {
        let home = std::env::var("HOME").unwrap();
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
