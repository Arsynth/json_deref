use super::path::AbsolutePath;
use serde_json::Value;
use std::collections::HashMap;

pub(crate) fn resolve_values(json: &Value, context: &HashMap<AbsolutePath, String>) -> Value {
    match json {
        Value::Object(map) => {
            let mut resolved_map = serde_json::Map::new();
            for (key, value) in map {
                resolved_map.insert(key.clone(), resolve_values(value, context));
            }
            Value::Object(resolved_map)
        }
        Value::Array(arr) => Value::Array(arr.iter().map(|v| resolve_values(v, context)).collect()),
        Value::String(text) => {
            let mut resolved_text = text.clone();
            let mut start_pos = 0;

            // Цикл по всем упоминаниям {path} в тексте
            while let Some(start) = resolved_text[start_pos..].find('{') {
                let absolute_start = start_pos + start;
                if let Some(end) = resolved_text[absolute_start..].find('}') {
                    let absolute_end = absolute_start + end;
                    let key_in_braces = &resolved_text[absolute_start + 1..absolute_end];

                    let absolute_path = AbsolutePath::new(key_in_braces);

                    if let Some(value) = context.get(&absolute_path) {
                        // Замена {path} на соответствующее разрешённое значение
                        resolved_text.replace_range(absolute_start..=absolute_end, value);
                        start_pos = absolute_start + value.len();
                    } else {
                        // Если путь не найден в контексте, продолжаем
                        start_pos = absolute_end + 1;
                    }
                } else {
                    break; // Если закрывающая скобка отсутствует, выходим
                }
            }
            Value::String(resolved_text)
        }
        _ => json.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::AbsolutePath;
    use super::*;
    use serde_json::json;
    use std::collections::HashMap;

    #[test]
    fn test_resolve_values_with_absolute_paths() {
        let json = json!({
            "config": {
                "level1": {
                    "key1": "value1",
                    "key2": "{/config/level1/key1}",
                    "key3": "{/config/level2/key4}"
                },
                "level2": {
                    "key4": "value4",
                    "key5": "{/config/level2/key4}"
                },
                "global_key": "{/config/level2/key4}"
            }
        });

        // Контекст с разрешёнными значениями
        let context = HashMap::from([
            (
                AbsolutePath::new("/config/level1/key1"),
                "value1".to_string(),
            ),
            (
                AbsolutePath::new("/config/level2/key4"),
                "value4".to_string(),
            ),
        ]);

        let resolved_json = resolve_values(&json, &context);

        let expected_resolved = json!({
            "config": {
                "level1": {
                    "key1": "value1",
                    "key2": "value1",
                    "key3": "value4"
                },
                "level2": {
                    "key4": "value4",
                    "key5": "value4"
                },
                "global_key": "value4"
            }
        });

        assert_eq!(resolved_json, expected_resolved);
    }

    #[test]
    fn test_resolve_values_with_missing_path() {
        let json = json!({
            "config": {
                "key1": "value1",
                "key2": "{/config/key3}" // Путь отсутствует в контексте
            }
        });

        // Контекст без /config/key3
        let context = HashMap::from([(AbsolutePath::new("/config/key1"), "value1".to_string())]);

        let resolved_json = resolve_values(&json, &context);

        // Ожидаемое поведение: отсутствующий путь остаётся нетронутым
        let expected_resolved = json!({
            "config": {
                "key1": "value1",
                "key2": "{/config/key3}"
            }
        });

        assert_eq!(resolved_json, expected_resolved);
    }

    #[test]
    fn test_resolve_values_simple_case() {
        let json = serde_json::json!({
            "posting_config": {
                "published_message_caption": "Link {/posting_config/invite_group_link} and {/posting_config/other_key}",
                "invite_group_link": "Invite Link",
                "other_key": "Other Value"
            }
        });

        // Контекст с разрешёнными значениями
        let context = HashMap::from([
            (
                AbsolutePath::new("/posting_config/invite_group_link"),
                "Invite Link".to_string(),
            ),
            (
                AbsolutePath::new("/posting_config/other_key"),
                "Other Value".to_string(),
            ),
        ]);

        let resolved_json = resolve_values(&json, &context);

        let expected_resolved = serde_json::json!({
            "posting_config": {
                "published_message_caption": "Link Invite Link and Other Value",
                "invite_group_link": "Invite Link",
                "other_key": "Other Value"
            }
        });

        assert_eq!(resolved_json, expected_resolved);
    }

    #[test]
    fn test_resolve_values_with_nested_dependencies() {
        let json = serde_json::json!({
            "config": {
                "level1": {
                    "key1": "value1",
                    "key2": "{/config/level1/key1}",
                    "key3": "{/config/level2/key4}"
                },
                "level2": {
                    "key4": "value4",
                    "key5": "{/config/level2/key4}"
                }
            }
        });

        // Контекст с разрешёнными значениями
        let context = HashMap::from([
            (
                AbsolutePath::new("/config/level1/key1"),
                "value1".to_string(),
            ),
            (
                AbsolutePath::new("/config/level2/key4"),
                "value4".to_string(),
            ),
            (
                AbsolutePath::new("/config/level2/key5"),
                "value4".to_string(),
            ),
        ]);

        let resolved_json = resolve_values(&json, &context);

        let expected_resolved = serde_json::json!({
            "config": {
                "level1": {
                    "key1": "value1",
                    "key2": "value1",
                    "key3": "value4"
                },
                "level2": {
                    "key4": "value4",
                    "key5": "value4"
                }
            }
        });

        assert_eq!(resolved_json, expected_resolved);
    }
}
