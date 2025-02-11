#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AbsolutePath(String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RelativePath(String);

impl AbsolutePath {
    /// Create a new absolute path from a string
    pub fn new(path: &str) -> Self {
        let normalized_path = Self::normalize(path);
        AbsolutePath(normalized_path)
    }

    /// Normalize the absolute path (remove extra slashes)
    fn normalize(path: &str) -> String {
        format!("/{}", path.trim_start_matches('/').trim_end_matches('/'))
    }

    /// Get the internal string
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Combine the current absolute path with a relative one
    pub fn resolve_with(&self, relative_path: &RelativePath) -> AbsolutePath {
        let mut base_parts: Vec<&str> = self.0.split('/').filter(|part| !part.is_empty()).collect();

        // Remove the current file component (if it's not the root)
        base_parts.pop();

        for segment in relative_path.0.split('/') {
            match segment {
                ".." => {
                    base_parts.pop();
                }
                "" => { /* Skip empty segments */ }
                _ => base_parts.push(segment),
            }
        }

        AbsolutePath(format!("/{}", base_parts.join("/")))
    }

    pub fn append(&self, path: &str) -> AbsolutePath {
        let result = format!(
            "{}/{}",
            self.0.trim_end_matches('/'),
            path.trim_start_matches('/').trim_end_matches('/')
        );
        AbsolutePath(result)
    }
}

impl Default for AbsolutePath {
    fn default() -> Self {
        Self::new("")
    }
}

impl RelativePath {
    /// Create a new relative path from a string
    pub fn new(path: &str) -> Self {
        RelativePath(path.to_string())
    }

    /// Get the internal string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_absolute_path_creation() {
        let abs_path = AbsolutePath::new("/some/nested/path/");
        assert_eq!(abs_path.as_str(), "/some/nested/path");
    }

    #[test]
    fn test_relative_path_creation() {
        let rel_path = RelativePath::new("../another/path");
        assert_eq!(rel_path.as_str(), "../another/path");
    }

    #[test]
    fn test_resolve_relative_path_to_absolute() {
        let base = AbsolutePath::new("/posting_config/nested/");
        let relative = RelativePath::new("../../invite_group_link");
        let resolved = base.resolve_with(&relative);
        assert_eq!(resolved.as_str(), "/invite_group_link");
    }

    #[test]
    fn test_resolve_relative_with_empty_segments() {
        let base = AbsolutePath::new("/a/b/c/");
        let relative = RelativePath::new("../../../d/e/");
        let resolved = base.resolve_with(&relative);
        assert_eq!(resolved.as_str(), "/d/e");
    }

    #[test]
    fn test_resolve_from_root() {
        let base = AbsolutePath::new("/");
        let relative = RelativePath::new("nested/path");
        let resolved = base.resolve_with(&relative);
        assert_eq!(resolved.as_str(), "/nested/path");
    }
}
