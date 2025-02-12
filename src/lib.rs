mod parsing;

use parsing::{
    collect_all_absolute_paths, expand_absolute_paths, extract_values_by_paths, make_deps_path_map, path::{AbsolutePath, RelativePath}, values_resolving::{resolve_recursive, resolve_values}
};
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::collections::{HashMap, HashSet};

/// Resolves JSON and returns a Value
pub fn resolve_json(input: &Value) -> Value {
    let mut path_map = HashMap::new();
    make_deps_path_map(input, &Default::default(), &mut path_map);

    let json_with_absolute_paths = expand_absolute_paths(input, &path_map, &Default::default());

    let path_maps: Vec<HashMap<RelativePath, AbsolutePath>> = path_map.values().cloned().collect();
    let mut paths = HashSet::new();
    for map in path_maps.iter() {
        paths.extend(map.values().cloned());
    }

    let mut extracted_values = HashMap::new();
    extract_values_by_paths(
        &json_with_absolute_paths,
        &paths,
        &Default::default(),
        &mut extracted_values,
    );

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

pub fn resolve_template_with_source(template: &Value, source: &Value) -> Value {
    // Build a HashMap for all absolute paths in the source JSON
    let mut source_map = HashMap::new();
    collect_all_absolute_paths(source, &AbsolutePath::new("/"), &mut source_map);

    resolve_recursive(template, &source_map)
}
