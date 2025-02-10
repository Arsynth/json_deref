#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AbsolutePath(String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RelativePath(String);

impl AbsolutePath {
    /// Создание нового абсолютного пути из строки
    pub fn new(path: &str) -> Self {
        let normalized_path = Self::normalize(path);
        AbsolutePath(normalized_path)
    }

    /// Нормализация абсолютного пути (удаление лишних слэшей)
    fn normalize(path: &str) -> String {
        format!("/{}", path.trim_start_matches('/').trim_end_matches('/'))
    }

    /// Получение внутренней строки
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Объединение текущего абсолютного пути с относительным
    pub fn resolve_with(&self, relative_path: &RelativePath) -> AbsolutePath {
        let mut base_parts: Vec<&str> = self.0.split('/').filter(|part| !part.is_empty()).collect();

        // Убираем текущий компонент файла (если это не корень)
        base_parts.pop();

        for segment in relative_path.0.split('/') {
            match segment {
                ".." => {
                    base_parts.pop();
                }
                "" => { /* Пропускаем пустые сегменты */ }
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
    /// Создание нового относительного пути из строки
    pub fn new(path: &str) -> Self {
        RelativePath(path.to_string())
    }

    /// Получение внутренней строки
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
