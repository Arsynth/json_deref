use super::path::AbsolutePath;
use serde_json::Value;
use std::collections::HashMap;

pub(crate) fn resolve_values(json: &Value, context: &HashMap<AbsolutePath, Value>) -> Value {
    match json {
        Value::Object(map) => {
            let mut resolved_map = serde_json::Map::new();
            for (key, value) in map {
                let resolved_value = resolve_values(value, context);
                resolved_map.insert(key.clone(), resolved_value);
            }
            Value::Object(resolved_map)
        }
        Value::Array(arr) => Value::Array(arr.iter().map(|v| resolve_values(v, context)).collect()),
        Value::String(text) => {
            // Check if the string is a single dependency like "{/absolute_path}"
            if let Some(dependency_path) = extract_dependency(&text) {
                // If it's a dependency, directly replace the value and type
                if let Some(resolved_value) = context.get(&dependency_path) {
                    return resolved_value.clone(); // Resolve to the dependency value and type
                }
            }

            // Handle embedded dependencies (e.g., "Hello {path}")
            resolve_embedded_refs(text, context)
        }
        _ => json.clone(), // Leave other types of JSON values untouched
    }
}

/// Extracts a dependency path from a string in the format "{path}".
/// Returns an AbsolutePath if the string is a valid single dependency, otherwise None.
fn extract_dependency(text: &str) -> Option<AbsolutePath> {
    // Check if the entire string is in the format "{dependency_path}"
    if text.starts_with('{') && text.ends_with('}') && text.len() > 2 {
        let path = &text[1..text.len() - 1]; // Extract the path between '{' and '}'
        return Some(AbsolutePath::new(path));
    }
    None
}

/// Resolves embedded references in a string, such as "Hello {path}".
/// This keeps the input as a string and replaces any "{dependency_path}" references within it.
fn resolve_embedded_refs(text: &str, context: &HashMap<AbsolutePath, Value>) -> Value {
    let mut resolved_text = text.to_string();
    let mut start_pos = 0;

    while let Some(start) = resolved_text[start_pos..].find('{') {
        let absolute_start = start_pos + start;
        if let Some(end) = resolved_text[absolute_start..].find('}') {
            let absolute_end = absolute_start + end;
            let path_in_braces = &resolved_text[absolute_start + 1..absolute_end];

            let absolute_path = AbsolutePath::new(path_in_braces);

            if let Some(resolved_value) = context.get(&absolute_path) {
                // Only replace embedded references if the resolved value is a string
                if let Value::String(resolved_string) = resolved_value {
                    resolved_text.replace_range(absolute_start..=absolute_end, resolved_string);
                    start_pos = absolute_start + resolved_string.len();
                } else {
                    // If the resolved value is not a string, return as untouched
                    return Value::String(text.to_string());
                }
            } else {
                // If no matching path is found, skip and move forward
                start_pos = absolute_end + 1;
            }
        } else {
            break;
        }
    }

    Value::String(resolved_text)
}

#[cfg(test)]
mod tests {
    use super::AbsolutePath;
    use super::*;
    use serde_json::json;
    use std::collections::HashMap;

    #[test]
    fn test_resolve_values_with_absolute_paths() {
        let json = serde_json::json!({
            "config": {
                "level1": {
                    "key1": "{/config/level1/key1_resolved}", // Simple reference to a string
                    "key2": "{/config/level1/key2_resolved}", // Reference to a number
                    "key3": "{/config/level2/key3_resolved}", // Reference to an array
                    "key4": "Embedded {/config/level2/key4_resolved} Example", // Embedded reference
                    "key5": "{/config/level2/key5_resolved}" // Reference to an object
                },
                "level2": {
                    "key3_resolved": [10, 20, 30], // Array
                    "key4_resolved": "value_here", // String to embed
                    "key5_resolved": { "nested_key": "nested_value" } // Object
                }
            }
        });

        // Context with predefined resolved values for absolute paths
        let context = HashMap::from([
            (
                AbsolutePath::new("/config/level1/key1_resolved"),
                Value::String("Resolved Value for Key1".to_string()),
            ),
            (
                AbsolutePath::new("/config/level1/key2_resolved"),
                Value::Number(123.into()), // Number
            ),
            (
                AbsolutePath::new("/config/level2/key3_resolved"),
                Value::Array(vec![
                    Value::Number(10.into()),
                    Value::Number(20.into()),
                    Value::Number(30.into()),
                ]),
            ),
            (
                AbsolutePath::new("/config/level2/key4_resolved"),
                Value::String("value_here".to_string()), // For embedded reference
            ),
            (
                AbsolutePath::new("/config/level2/key5_resolved"),
                Value::Object(serde_json::Map::from_iter(vec![(
                    "nested_key".to_string(),
                    Value::String("nested_value".to_string()),
                )])),
            ),
        ]);

        let resolved_json = resolve_values(&json, &context);

        // Expected JSON after resolving all dependencies
        let expected_resolved = serde_json::json!({
            "config": {
                "level1": {
                    "key1": "Resolved Value for Key1",
                    "key2": 123,
                    "key3": [10, 20, 30],
                    "key4": "Embedded value_here Example",
                    "key5": { "nested_key": "nested_value" }
                },
                "level2": {
                    "key3_resolved": [10, 20, 30],
                    "key4_resolved": "value_here",
                    "key5_resolved": { "nested_key": "nested_value" }
                }
            }
        });

        assert_eq!(resolved_json, expected_resolved);
    }

    #[test]
    fn test_resolve_values_with_missing_path() {
        let json = serde_json::json!({
            "config": {
                "key1": "value1", // Regular value
                "key2": "{/config/key3}", // Reference to a missing path
                "key3": 42, // Existing value, but not used in the context
                "key4": "Hello {/missing/path}" // Embedded reference to a missing path
            }
        });

        // Context that omits some paths intentionally
        let context = HashMap::from([
            // Only define `/config/key1` in the context
            (
                AbsolutePath::new("/config/key1"),
                Value::String("value1".to_string()),
            ),
        ]);

        // Resolve the values in the JSON based on the context
        let resolved_json = resolve_values(&json, &context);

        // Expected JSON after resolving what is resolvable
        let expected_resolved = serde_json::json!({
            "config": {
                "key1": "value1", // Resolved correctly
                "key2": "{/config/key3}", // Missing path remains unresolved
                "key3": 42, // Key untouched as it is not referenced
                "key4": "Hello {/missing/path}" // Embedded missing path remains unresolved
            }
        });

        // Assert that the resolved JSON matches the expected result
        assert_eq!(resolved_json, expected_resolved);
    }

    #[test]
    fn test_resolve_values_simple_case() {
        let json = serde_json::json!({
            "posting_config": {
                "published_message_caption": "Link {/posting_config/invite_group_link} and {/posting_config/other_key}",
                "invite_group_link": "This should be replaced",
                "other_key": "This too"
            }
        });

        // Context containing paths to resolve and their corresponding values
        let context = HashMap::from([
            (
                AbsolutePath::new("/posting_config/invite_group_link"),
                Value::String("Invite Link".to_string()),
            ),
            (
                AbsolutePath::new("/posting_config/other_key"),
                Value::String("Other Value".to_string()),
            ),
        ]);

        // Perform resolution
        let resolved_json = resolve_values(&json, &context);

        // Expected JSON after resolving the references
        let expected_resolved = serde_json::json!({
            "posting_config": {
                "published_message_caption": "Link Invite Link and Other Value",
                "invite_group_link": "This should be replaced",
                "other_key": "This too"
            }
        });

        // Assert that the resolved JSON matches the expected result
        assert_eq!(resolved_json, expected_resolved);
    }

    #[test]
    fn test_resolve_values_with_nested_dependencies() {
        let json = serde_json::json!({
            "config": {
                "level1": {
                    "key1": "{/config/level2/key4}", // Ссылка на числовое значение
                    "key2": "{/config/level2/key5}", // Ссылка на массив
                    "key3": "{/config/level2/key6}", // Ссылка на объект
                    "key4": "Hello {/config/level2/key7}" // Вложенная ссылка в строке
                },
                "level2": {
                    "key4": 100,
                    "key5": [1, 2, 3],
                    "key6": {"nested_key": "nested_value"},
                    "key7": "World"
                }
            }
        });

        // Контекст с разрешёнными значениями (разных типов)
        let context = HashMap::from([
            (
                AbsolutePath::new("/config/level2/key4"),
                Value::Number(100.into()),
            ),
            (
                AbsolutePath::new("/config/level2/key5"),
                Value::Array(vec![
                    Value::Number(1.into()),
                    Value::Number(2.into()),
                    Value::Number(3.into()),
                ]),
            ),
            (
                AbsolutePath::new("/config/level2/key6"),
                Value::Object(serde_json::Map::from_iter(vec![(
                    "nested_key".to_string(),
                    Value::String("nested_value".to_string()),
                )])),
            ),
            (
                AbsolutePath::new("/config/level2/key7"),
                Value::String("World".to_string()),
            ),
        ]);

        let resolved_json = resolve_values(&json, &context);

        let expected_resolved = serde_json::json!({
            "config": {
                "level1": {
                    "key1": 100,
                    "key2": [1, 2, 3],
                    "key3": {"nested_key": "nested_value"},
                    "key4": "Hello World"
                },
                "level2": {
                    "key4": 100,
                    "key5": [1, 2, 3],
                    "key6": {"nested_key": "nested_value"},
                    "key7": "World"
                }
            }
        });

        assert_eq!(resolved_json, expected_resolved);
    }
}
