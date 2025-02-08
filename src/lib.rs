use serde::de::DeserializeOwned;
use serde_json::{Map, Value};
use std::collections::{HashMap, HashSet};

/// Resolves JSON and returns a Value
pub fn resolve_json(input: &Value) -> Value {
    let mut path_map = HashMap::new();
    prepare_path_map(input, "", &mut path_map);

    let json_with_absolute_paths = convert_to_absolute_paths(input, &path_map);

    let paths: HashSet<String> = path_map.values().cloned().collect();

    let mut extracted_values = HashMap::new();
    extract_values_from_paths(&json_with_absolute_paths, &paths, "", &mut extracted_values);

    resolve_values(&json_with_absolute_paths, &extracted_values)
}

/// Resolves JSON from Value and returns generic object
pub fn resolve_json_to_object<T>(input: &Value) -> Result<T, serde_json::Error>
where
    T: DeserializeOwned,
{
    let resolved_json = resolve_json(input);
    serde_json::from_value(resolved_json)
}

/// Resolves JSON from Read and returns generic object
pub fn resolve_json_reader_to_object<R, T>(reader: R) -> Result<T, serde_json::Error>
where
    R: std::io::Read,
    T: DeserializeOwned,
{
    let input = serde_json::from_reader(reader)?;
    resolve_json_to_object(&input)
}

fn resolve_reference_path(base_path: &str, relative_path: &str) -> String {
    let mut base_parts: Vec<&str> = base_path
        .split('/')
        .filter(|part| !part.is_empty())
        .collect();

    base_parts.pop();

    for segment in relative_path.split('/') {
        match segment {
            ".." => {
                base_parts.pop();
            }
            "" => { /* ignore empty segments */ }
            _ => base_parts.push(segment), 
        }
    }

    format!("/{}", base_parts.join("/"))
}

fn collect_absolute_paths(json: &Value, base_path: &str, context: &mut HashMap<String, String>) {
    match json {
        Value::Object(map) => {
            for (key, value) in map {
                let current_path = format!("{}/{}", base_path, key);
                collect_absolute_paths(value, &current_path, context);
            }
        }
        Value::String(text) => {
            context.insert(base_path.to_string(), text.clone());
        }
        Value::Number(number) => {
            context.insert(base_path.to_string(), number.to_string());
        }
        _ => {}
    }
}

fn prepare_path_map(json: &Value, base_path: &str, path_map: &mut HashMap<String, String>) {
    match json {
        Value::Object(map) => {
            for (key, value) in map {
                let current_absolute_path = format!("{}/{}", base_path, key);
                prepare_path_map(value, &current_absolute_path, path_map);
            }
        }
        Value::Array(arr) => {
            for value in arr {
                prepare_path_map(value, base_path, path_map);
            }
        }
        Value::String(text) => {
            let mut start_pos = 0;
            while let Some(start) = text[start_pos..].find('{') {
                let absolute_start = start_pos + start; 
                if let Some(end) = text[absolute_start..].find('}') {
                    let absolute_end = absolute_start + end; 
                    let reference = &text[absolute_start + 1..absolute_end];

                    if reference.starts_with('/') {
                        path_map.insert(reference.to_string(), reference.to_string());
                        start_pos = absolute_end + 1; 
                        continue;
                    }

                    let absolute_path = resolve_reference_path(base_path, reference);
                    path_map.insert(reference.to_string(), absolute_path);

                    start_pos = absolute_end + 1; 
                } else {
                    break;
                }
            }
        }
        _ => {}
    }
}

fn convert_to_absolute_paths(json: &Value, path_map: &HashMap<String, String>) -> Value {
    match json {
        Value::Object(map) => {
            let mut new_map = Map::new();
            for (key, value) in map {
                new_map.insert(key.clone(), convert_to_absolute_paths(value, path_map));
            }
            Value::Object(new_map)
        }
        Value::Array(arr) => {
            Value::Array(
                arr.iter()
                    .map(|v| convert_to_absolute_paths(v, path_map))
                    .collect(),
            )
        }
        Value::String(text) => {
            let mut updated_text = text.clone();
            let mut start_pos = 0;
            while let Some(start) = updated_text[start_pos..].find('{') {
                let absolute_start = start_pos + start;
                if let Some(end) = updated_text[absolute_start..].find('}') {
                    let absolute_end = absolute_start + end;
                    let relative_key = &updated_text[absolute_start + 1..absolute_end];
                                                                                        
                    if let Some(absolute_path) = path_map.get(relative_key) {

                        updated_text.replace_range(
                            absolute_start..=absolute_end,
                            &format!("{{{}}}", absolute_path),
                        );
                        start_pos = absolute_start + absolute_path.len();
                    } else {
                        start_pos = absolute_end + 1;
                    }
                } else {
                    break;
                }
            }
            Value::String(updated_text)
        }
        _ => json.clone(),
    }
}

fn extract_values_from_paths(
    json: &Value,
    paths: &HashSet<String>,
    current_path: &str,
    extracted_values: &mut HashMap<String, String>,
) {
    match json {
        Value::Object(map) => {
            for (key, value) in map {
                let new_path = format!("{}/{}", current_path, key); 
                extract_values_from_paths(value, paths, &new_path, extracted_values);
            }
        }
        Value::Array(arr) => {
            for (index, value) in arr.iter().enumerate() {
                let new_path = format!("{}/{}", current_path, index); 
                extract_values_from_paths(value, paths, &new_path, extracted_values);
            }
        }
        Value::String(text) => {
            if paths.contains(current_path) {
                extracted_values.insert(current_path.to_string(), text.clone());
            }
        }
        Value::Number(num) => {
            if paths.contains(current_path) {
                extracted_values.insert(current_path.to_string(), num.to_string());
            }
        }
        _ => {} 
    }
}

fn resolve_values(json: &Value, context: &HashMap<String, String>) -> Value {
    match json {
        Value::Object(map) => {
            let mut resolved_map = Map::new();
            for (key, value) in map {
                resolved_map.insert(key.clone(), resolve_values(value, context));
            }
            Value::Object(resolved_map)
        }
        Value::Array(arr) => Value::Array(arr.iter().map(|v| resolve_values(v, context)).collect()),
        Value::String(text) => {
            let mut resolved_text = text.clone();
            let mut start_pos = 0; 
            while let Some(start) = resolved_text[start_pos..].find('{') {
                let absolute_start = start_pos + start; 
                if let Some(end) = resolved_text[absolute_start..].find('}') {
                    let absolute_end = absolute_start + end;
                    let absolute_key = &resolved_text[absolute_start + 1..absolute_end];

                    if let Some(value) = context.get(absolute_key) {
                        resolved_text.replace_range(absolute_start..=absolute_end, value);
                        start_pos = absolute_start + value.len();
                    } else {
                        start_pos = absolute_end + 1;
                    }
                } else {
                    break;
                }
            }
            Value::String(resolved_text)
        }
        _ => json.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use serde_json::json;

    #[test]
    fn test_resolve_reference_path() {
        assert_eq!(
            resolve_reference_path("/posting_config", "../invite_group_link"),
            "/invite_group_link"
        );
        assert_eq!(
            resolve_reference_path("/posting_config/nested", "../../invite_group_link"),
            "/invite_group_link"
        );
        assert_eq!(
            resolve_reference_path("/posting_config", "invite_group_link"),
            "/invite_group_link"
        );
        assert_eq!(
            resolve_reference_path("", "invite_group_link"),
            "/invite_group_link"
        );
    }

    #[test]
    fn test_collect_absolute_paths() {
        let input = json!({
            "key1": "value1",
            "key2": {
                "nested1": "value2",
                "nested2": {
                    "deep": "value3"
                }
            }
        });

        let mut context = HashMap::new();
        collect_absolute_paths(&input, "", &mut context);

        assert_eq!(context.get("/key1"), Some(&"value1".to_string()));
        assert_eq!(context.get("/key2/nested1"), Some(&"value2".to_string()));
        assert_eq!(
            context.get("/key2/nested2/deep"),
            Some(&"value3".to_string())
        );
    }

    #[test]
    fn test_convert_to_absolute_paths() {
        let input = json!({
            "posting_config": {
                "published_message_caption": "Check this {../invite_group_link} or {invite_group_link}",
                "invite_group_link": "link_value"
            }
        });

        let mut path_map = HashMap::new();
        prepare_path_map(&input, "", &mut path_map);

        let result = convert_to_absolute_paths(&input, &path_map);
        let expected = json!({
            "posting_config": {
                "published_message_caption": "Check this {/invite_group_link} or {/posting_config/invite_group_link}",
                "invite_group_link": "link_value"
            }
        });

        assert_eq!(result, expected);
    }

    #[test]
    fn test_resolve_values() {
        let input = json!({
            "posting_config": {
                "published_message_caption": "Text with {/posting_config/invite_group_link}",
                "invite_group_link": "link_value"
            }
        });

        let mut context = HashMap::new();
        collect_absolute_paths(&input, "", &mut context);

        let result = resolve_values(&input, &context);
        let expected = json!({
            "posting_config": {
                "published_message_caption": "Text with link_value",
                "invite_group_link": "link_value"
            }
        });

        assert_eq!(result, expected);
    }

    #[test]
    fn test_resolve_json_with_dependencies_at_different_levels() {
        let input = json!({
            "config": {
                "level1": {
                    "key1": "value1",
                    "key2": "{key1}",
                    "nested": {
                        "key3": "{../../level2/key4}",
                        "key4": "local_value"
                    }
                },
                "level2": {
                    "key4": "value4",
                    "key5": "{key4}" 
                },
                "global_key": "{/config/level2/key4}",
                "global_dependency": "{/config/level1/nested/key4}" 
            }
        });

        let mut path_map = HashMap::new();
        prepare_path_map(&input, "", &mut path_map);

        assert_eq!(
            path_map.get("key1"),
            Some(&"/config/level1/key1".to_string())
        );
        assert_eq!(
            path_map.get("../../level2/key4"),
            Some(&"/config/level2/key4".to_string())
        );
        assert_eq!(
            path_map.get("key4"),
            Some(&"/config/level2/key4".to_string())
        ); 

        let json_absolute = convert_to_absolute_paths(&input, &path_map);
        let expected_absolute = json!({
            "config": {
                "level1": {
                    "key1": "value1",
                    "key2": "{/config/level1/key1}",
                    "nested": {
                        "key3": "{/config/level2/key4}",
                        "key4": "local_value"
                    }
                },
                "level2": {
                    "key4": "value4",
                    "key5": "{/config/level2/key4}"
                },
                "global_key": "{/config/level2/key4}",
                "global_dependency": "{/config/level1/nested/key4}"
            }
        });
        assert_eq!(json_absolute, expected_absolute);

        let mut values_map = HashMap::new();
        let paths: HashSet<String> = path_map.values().cloned().collect();
        extract_values_from_paths(&json_absolute, &paths, "", &mut values_map);

        let resolved_json = resolve_values(&json_absolute, &values_map);
        let expected_resolved = json!({
            "config": {
                "level1": {
                    "key1": "value1",
                    "key2": "value1",
                    "nested": {
                        "key3": "value4",
                        "key4": "local_value"
                    }
                },
                "level2": {
                    "key4": "value4",
                    "key5": "value4"
                },
                "global_key": "value4",
                "global_dependency": "local_value"
            }
        });
        assert_eq!(resolved_json, expected_resolved);
    }

    #[test]
    fn test_resolve_values_simple_case() {
        let json = json!({
            "posting_config": {
                "published_message_caption": "Link {invite_group_link} and {other_key}",
                "invite_group_link": "Invite Link",
                "other_key": "Other Value"
            }
        });

        let mut path_map = HashMap::new();
        path_map.insert("invite_group_link".to_string(), "Invite Link".to_string());
        path_map.insert("other_key".to_string(), "Other Value".to_string());

        let expected_resolved = json!({
            "posting_config": {
                "published_message_caption": "Link Invite Link and Other Value",
                "invite_group_link": "Invite Link",
                "other_key": "Other Value"
            }
        });

        let resolved = resolve_values(&json, &path_map);
        assert_eq!(resolved, expected_resolved);
    }

    #[test]
    fn test_resolve_values_with_nested_dependencies() {
        let json = json!({
            "config": {
                "level1": {
                    "key1": "value1",
                    "key2": "{key1}",
                    "key3": "{/config/level2/key4}"
                },
                "level2": {
                    "key4": "value4",
                    "key5": "{key4}"
                }
            }
        });

        let mut path_map = HashMap::new();
        path_map.insert("key1".to_string(), "value1".to_string());
        path_map.insert("/config/level2/key4".to_string(), "value4".to_string());
        path_map.insert("key4".to_string(), "value4".to_string());

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
                }
            }
        });

        let resolved = resolve_values(&json, &path_map);
        assert_eq!(resolved, expected_resolved);
    }

    /// Test will fail. Multiple absolute paths are not supported yet.
    #[test]
    fn test_relative_path_with_multiple_absolute_resolutions() {
        let input = serde_json::json!({
            "branch1": {
                "parent_key": "value1",
                "subbranch": {
                    "child_key": "{../parent_key}"
                }
            },
            "branch2": {
                "parent_key": "value2",
                "subbranch": {
                    "child_key": "{../parent_key}"
                }
            }
        });

        let mut path_map = HashMap::new();
        prepare_path_map(&input, "", &mut path_map);

        println!("Path Map: {:#?}", path_map);
        assert_eq!(
            path_map.get("../parent_key"),
            Some(&"/branch1/parent_key".to_string()) 
        );
        assert_eq!(
            path_map.get("../parent_key"),
            Some(&"/branch2/parent_key".to_string())
        );

        let json_with_absolute_paths = convert_to_absolute_paths(&input, &path_map);
        println!("JSON with Absolute Paths:\n{}", json_with_absolute_paths);

        let expected_json_with_absolute_paths = serde_json::json!({
            "branch1": {
                "parent_key": "value1",
                "subbranch": {
                    "child_key": "{/branch1/parent_key}"
                }
            },
            "branch2": {
                "parent_key": "value2",
                "subbranch": {
                    "child_key": "{/branch2/parent_key}"
                }
            }
        });
        assert_eq!(json_with_absolute_paths, expected_json_with_absolute_paths);

        let paths: HashSet<String> = path_map.values().cloned().collect();
        let mut extracted_values = HashMap::new();
        extract_values_from_paths(&json_with_absolute_paths, &paths, "", &mut extracted_values);

        println!("Extracted Values: {:#?}", extracted_values);
        assert_eq!(
            extracted_values.get("/branch1/parent_key"),
            Some(&"value1".to_string())
        );
        assert_eq!(
            extracted_values.get("/branch2/parent_key"),
            Some(&"value2".to_string())
        );

        let resolved_json = resolve_values(&json_with_absolute_paths, &extracted_values);
        println!(
            "Resolved JSON:\n{}",
            serde_json::to_string_pretty(&resolved_json).unwrap()
        );

        let expected_resolved_json = serde_json::json!({
            "branch1": {
                "parent_key": "value1",
                "subbranch": {
                    "child_key": "value1"
                }
            },
            "branch2": {
                "parent_key": "value2",
                "subbranch": {
                    "child_key": "value2"
                }
            }
        });
        assert_eq!(resolved_json, expected_resolved_json);
    }

    #[test]
    fn test_resolve_json_to_object_using_hashset() {
        let input = serde_json::json!({
            "branch1": {
                "parent_key": "value1",
                "subbranch": {
                    "child_key": "{../parent_key}"
                }
            },
            "branch2": {
                "parent_key": "value2",
                "parent_key2": "value22",
                "subbranch": {
                    "child_key": "{../parent_key2}"
                }
            }
        });

        #[derive(Debug, Deserialize, PartialEq)]
        struct Branch {
            parent_key: String,
            subbranch: Subbranch,
        }

        #[derive(Debug, Deserialize, PartialEq)]
        struct Subbranch {
            child_key: String,
        }

        #[derive(Debug, Deserialize, PartialEq)]
        struct Root {
            branch1: Branch,
            branch2: Branch,
        }

        let result: Result<Root, _> = resolve_json_to_object(&input);

        assert!(result.is_ok());

        let resolved = result.unwrap();
        let expected = Root {
            branch1: Branch {
                parent_key: "value1".to_string(),
                subbranch: Subbranch {
                    child_key: "value1".to_string(),
                },
            },
            branch2: Branch {
                parent_key: "value2".to_string(),
                subbranch: Subbranch {
                    child_key: "value22".to_string(),
                },
            },
        };

        assert_eq!(resolved, expected);
    }
}
