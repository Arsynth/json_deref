pub(crate) mod path;
pub(crate) mod values_resolving;

use path::{AbsolutePath, RelativePath};
use serde_json::{Map, Value};
use std::collections::{HashMap, HashSet};

pub(crate) fn make_deps_path_map(
    json: &Value,
    base_path: &AbsolutePath,
    complete_path_map: &mut HashMap<AbsolutePath, HashMap<RelativePath, AbsolutePath>>,
) {
    match json {
        Value::Object(map) => {
            for (key, value) in map {
                let current_absolute_path = base_path.append(key.as_str());
                make_deps_path_map(value, &current_absolute_path, complete_path_map);
            }
        }
        Value::Array(arr) => {
            for (i, value) in arr.iter().enumerate() {
                let current_absolute_path = base_path.append(&format!("{i}"));
                make_deps_path_map(value, &current_absolute_path, complete_path_map);
            }
        }
        Value::String(text) => {
            let mut dependencies = HashMap::new();
            let mut start_pos = 0;
            while let Some(start) = text[start_pos..].find('{') {
                let absolute_start = start_pos + start;
                if let Some(end) = text[absolute_start..].find('}') {
                    let absolute_end = absolute_start + end;
                    let reference = &text[absolute_start + 1..absolute_end];

                    // Determine whether the reference is relative or absolute
                    let relative_path = RelativePath::new(reference);
                    if reference.starts_with('/') {
                        dependencies.insert(relative_path.clone(), AbsolutePath::new(reference));
                    } else {
                        let absolute_dependency_path = base_path.resolve_with(&relative_path);
                        dependencies.insert(relative_path, absolute_dependency_path);
                    }
                    start_pos = absolute_end + 1;
                } else {
                    break;
                }
            }

            // Only insert into `complete_path_map` if there are actual dependencies
            if !dependencies.is_empty() {
                complete_path_map.insert(base_path.clone(), dependencies);
            }
        }
        _ => {} // Ignore other types like numbers, booleans, or nulls
    }
}

pub(crate) fn expand_absolute_paths(
    json: &Value,
    path_map: &HashMap<AbsolutePath, HashMap<RelativePath, AbsolutePath>>,
    current_path: &AbsolutePath,
) -> Value {
    match json {
        Value::Object(map) => {
            let mut new_map = Map::new();
            for (key, value) in map {
                let new_path = current_path.append(&key);
                new_map.insert(
                    key.clone(),
                    expand_absolute_paths(value, path_map, &new_path),
                );
            }
            Value::Object(new_map)
        }
        Value::Array(arr) => Value::Array(
            arr.iter()
                .enumerate()
                .map(|(i, v)| {
                    let new_path = current_path.append(&format!("{i}"));
                    expand_absolute_paths(v, path_map, &new_path)
                })
                .collect(),
        ),
        Value::String(text) => {
            let mut updated_text = text.clone();
            let mut start_pos = 0;

            while let Some(start) = updated_text[start_pos..].find('{') {
                let absolute_start = start_pos + start;
                if let Some(end) = updated_text[absolute_start..].find('}') {
                    let absolute_end = absolute_start + end;
                    let relative_key =
                        &RelativePath::new(&updated_text[absolute_start + 1..absolute_end]);

                    if let Some(dependencies) = path_map.get(current_path) {
                        if let Some(absolute_path) = dependencies.get(relative_key) {
                            updated_text.replace_range(
                                absolute_start..=absolute_end,
                                &format!("{{{}}}", absolute_path.as_str()),
                            );
                            start_pos = absolute_start + absolute_path.as_str().len() + 2;
                        } else {
                            start_pos = absolute_end + 1;
                        }
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

pub(crate) fn extract_values_by_paths(
    json: &Value,
    paths: &HashSet<AbsolutePath>,
    current_path: &AbsolutePath,
    extracted_values: &mut HashMap<AbsolutePath, Value>,
) {
    match json {
        Value::Object(map) => {
            // If the entire object itself is in the paths to extract, add it
            if paths.contains(current_path) {
                extracted_values.insert(current_path.clone(), json.clone());
            }

            // Recurse into the map to explore nested paths
            for (key, value) in map {
                let new_path = current_path.append(key);
                extract_values_by_paths(value, paths, &new_path, extracted_values);
            }
        }
        Value::Array(arr) => {
            for (index, value) in arr.iter().enumerate() {
                // Create a new path for the array index
                let new_path = current_path.append(&index.to_string());
                // Recurse into the array
                extract_values_by_paths(value, paths, &new_path, extracted_values);
            }
            // Check if the entire array itself is part of the `paths` to extract
            if paths.contains(current_path) {
                extracted_values.insert(current_path.clone(), json.clone());
            }
        }
        _ => {
            // If the current path is in the set of paths to extract, add it to the map
            if paths.contains(current_path) {
                extracted_values.insert(current_path.clone(), json.clone());
            }
        }
    }
}

pub(crate) fn collect_all_absolute_paths(
    json: &Value,
    current_path: &AbsolutePath,
    source_map: &mut HashMap<AbsolutePath, Value>,
) {
    match json {
        Value::Object(map) => {
            for (key, value) in map {
                let new_path = current_path.append(key);
                source_map.insert(new_path.clone(), value.clone());
                collect_all_absolute_paths(value, &new_path, source_map);
            }
        }
        Value::Array(array) => {
            for (index, value) in array.iter().enumerate() {
                let new_path = current_path.append(&index.to_string());
                source_map.insert(new_path.clone(), value.clone());
                collect_all_absolute_paths(value, &new_path, source_map);
            }
        }
        _ => {
            source_map.insert(current_path.clone(), json.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::parsing::values_resolving::resolve_values;

    use super::*;
    use serde_json::json;

    #[test]
    fn test_convert_to_absolute_paths_with_one_invalid() {
        let input = json!({
            "posting_config": {
                "published_message_caption": "Check this {../invite_group_link} or {invite_group_link}",
                "invite_group_link": "link_value"
            }
        });

        let mut path_map = HashMap::new();
        make_deps_path_map(&input, &Default::default(), &mut path_map);

        let result = expand_absolute_paths(&input, &path_map, &Default::default());
        let expected = json!({
            "posting_config": {
                "published_message_caption": "Check this {/invite_group_link} or {/posting_config/invite_group_link}",
                "invite_group_link": "link_value"
            }
        });

        assert_eq!(result, expected);
    }

    #[test]
    fn test_resolve_json_with_dependencies_at_different_levels() {
        let input = serde_json::json!({
            "config": {
                "level1": {
                    "key1": "value1", // Simple string
                    "key2": "{/config/level1/key1}", // Reference to another entry
                    "key3": "{/config/level2/key4}", // Reference to a number
                    "nested": {
                        "key4": "{../../level2/key5}", // Relative reference to an array
                        "key5": "local_value" // Simple local string
                    }
                },
                "level2": {
                    "key4": 42, // A number
                    "key5": [1, 2, 3], // An array
                    "key6": "{/config/level1/nested/key5}" // Reference to a local level1 string
                },
                "global_key": "{/config/level2/key4}", // Global reference to a number in level2
                "global_dependency": "{/config/level1/nested/key5}" // Global reference to a string in level1
            }
        });

        // Generate the `path_map` using `make_deps_path_map`
        let mut path_map = HashMap::new();
        make_deps_path_map(&input, &AbsolutePath::new("/"), &mut path_map);

        // The corrected `expected_path_map`
        let expected_path_map = HashMap::from([
            (
                AbsolutePath::new("/config/global_key"),
                HashMap::from([(
                    RelativePath::new("/config/level2/key4"),
                    AbsolutePath::new("/config/level2/key4"),
                )]),
            ),
            (
                AbsolutePath::new("/config/global_dependency"),
                HashMap::from([(
                    RelativePath::new("/config/level1/nested/key5"),
                    AbsolutePath::new("/config/level1/nested/key5"),
                )]),
            ),
            (
                AbsolutePath::new("/config/level1/key2"),
                HashMap::from([(
                    RelativePath::new("/config/level1/key1"),
                    AbsolutePath::new("/config/level1/key1"),
                )]),
            ),
            (
                AbsolutePath::new("/config/level1/key3"),
                HashMap::from([(
                    RelativePath::new("/config/level2/key4"),
                    AbsolutePath::new("/config/level2/key4"),
                )]),
            ),
            (
                AbsolutePath::new("/config/level1/nested/key4"),
                HashMap::from([(
                    RelativePath::new("../../level2/key5"),
                    AbsolutePath::new("/config/level2/key5"),
                )]),
            ),
            (
                AbsolutePath::new("/config/level2/key6"),
                HashMap::from([(
                    RelativePath::new("/config/level1/nested/key5"),
                    AbsolutePath::new("/config/level1/nested/key5"),
                )]),
            ),
        ]);

        // Validate the generated `path_map` matches the expectation
        assert_eq!(path_map, expected_path_map);

        // Ensure `expand_absolute_paths` resolves dependencies
        let result_json = expand_absolute_paths(&input, &path_map, &AbsolutePath::new("/"));
        let expected_json = serde_json::json!({
            "config": {
                "level1": {
                    "key1": "value1",
                    "key2": "{/config/level1/key1}",
                    "key3": "{/config/level2/key4}",
                    "nested": {
                        "key4": "{/config/level2/key5}",
                        "key5": "local_value"
                    }
                },
                "level2": {
                    "key4": 42,
                    "key5": [1, 2, 3],
                    "key6": "{/config/level1/nested/key5}"
                },
                "global_key": "{/config/level2/key4}",
                "global_dependency": "{/config/level1/nested/key5}"
            }
        });

        // Validate the resolved JSON matches the expected structure
        assert_eq!(result_json, expected_json);

        // Extract values for paths based on `path_map`
        let mut extracted_values = HashMap::new();
        let absolute_paths: HashSet<AbsolutePath> = path_map
            .values()
            .flat_map(|map| map.values().cloned())
            .collect();
        extract_values_by_paths(
            &result_json,
            &absolute_paths,
            &AbsolutePath::new("/"),
            &mut extracted_values,
        );

        // Verify the extracted values for each referred path
        let expected_extracted_values = HashMap::from([
            (
                AbsolutePath::new("/config/level2/key4"),
                Value::Number(42.into()), // This resolves to 42
            ),
            (
                AbsolutePath::new("/config/level2/key5"),
                Value::Array(vec![
                    Value::Number(1.into()),
                    Value::Number(2.into()),
                    Value::Number(3.into()),
                ]), // This resolves to an array
            ),
            (
                AbsolutePath::new("/config/level1/nested/key5"),
                Value::String("local_value".to_string()), // This resolves to local_value
            ),
            (
                AbsolutePath::new("/config/level1/key1"),
                Value::String("value1".to_string()), // This resolves to value1
            ),
        ]);
        assert_eq!(extracted_values, expected_extracted_values);

        // Lastly, fully resolve the JSON using `resolve_values`
        let resolved_json = resolve_values(&result_json, &extracted_values);
        let expected_resolved = serde_json::json!({
            "config": {
                "level1": {
                    "key1": "value1",
                    "key2": "value1",
                    "key3": 42,
                    "nested": {
                        "key4": [1, 2, 3],
                        "key5": "local_value"
                    }
                },
                "level2": {
                    "key4": 42,
                    "key5": [1, 2, 3],
                    "key6": "local_value"
                },
                "global_key": 42,
                "global_dependency": "local_value"
            }
        });

        // Validate the final resolved JSON
        assert_eq!(resolved_json, expected_resolved);
    }

    #[test]
    fn test_extract_values_by_paths() {
        let input = serde_json::json!({
            "config": {
                "level1": {
                    "key1": "value1", // String
                    "key2": 42,       // Number
                    "key3": [1, 2, 3], // Array
                },
                "level2": {
                    "key4": { "nested_key": "nested_value" }, // Object
                    "key5": true, // Boolean
                    "key6": null  // Null
                }
            }
        });

        // Define the paths for which values should be extracted
        let paths_to_extract = HashSet::from([
            AbsolutePath::new("/config/level1/key1"), // String
            AbsolutePath::new("/config/level1/key2"), // Number
            AbsolutePath::new("/config/level1/key3"), // Array
            AbsolutePath::new("/config/level2/key4"), // Object
            AbsolutePath::new("/config/level2/key5"), // Boolean
            AbsolutePath::new("/config/level2/key6"), // Null
        ]);

        let mut extracted_values = HashMap::new();

        // Perform extraction
        extract_values_by_paths(
            &input,
            &paths_to_extract,
            &AbsolutePath::new("/"), // Start from the root
            &mut extracted_values,
        );

        // Define expected extracted values (with original types)
        let expected_extracted_values = HashMap::from([
            (
                AbsolutePath::new("/config/level1/key1"),
                Value::String("value1".to_string()),
            ), // String
            (
                AbsolutePath::new("/config/level1/key2"),
                Value::Number(42.into()),
            ), // Number
            (
                AbsolutePath::new("/config/level1/key3"),
                Value::Array(vec![
                    Value::Number(1.into()),
                    Value::Number(2.into()),
                    Value::Number(3.into()),
                ]), // Array
            ),
            (
                AbsolutePath::new("/config/level2/key4"),
                Value::Object(serde_json::Map::from_iter(vec![(
                    "nested_key".to_string(),
                    Value::String("nested_value".to_string()),
                )])), // Object
            ),
            (AbsolutePath::new("/config/level2/key5"), Value::Bool(true)), // Boolean
            (AbsolutePath::new("/config/level2/key6"), Value::Null),       // Null
        ]);

        // Assert that the extracted values match the expected values
        assert_eq!(extracted_values, expected_extracted_values);
    }
}
