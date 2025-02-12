mod parsing;

use parsing::{
    collect_all_absolute_paths, expand_absolute_paths, extract_values_by_paths, make_deps_path_map,
    path::{AbsolutePath, RelativePath},
    values_resolving::{resolve_recursive, resolve_values},
};
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::collections::{HashMap, HashSet};

/// A trait to add convenient JSON template and resolution methods for serde_json::Value.
pub trait JsonResolvableFunctions {
    /// Resolve JSON placeholders within itself.
    fn resolve_internal_dependencies(&self) -> Value;

    /// Resolve the JSON as a template using another JSON as a source.
    ///
    /// - `source`: The source JSON containing the values for placeholders.
    fn resolve_template_with_source(&self, source: &Value) -> Value;
}

impl JsonResolvableFunctions for Value {
    /// Resolves internal dependencies within the JSON object.
    ///
    /// This uses the JSON itself as the source for resolving placeholders. Placeholders
    /// can reference fields in the JSON using either:
    /// - **Absolute paths**: `{/path/to/value}`
    /// - **Relative paths**:
    ///   - `{field_name}`: Refers to sibling fields in the same object.
    ///   - `{../../parent_field}`: Refers to fields higher up in the hierarchy.
    ///
    /// If a placeholder cannot be resolved (e.g., nonexistent paths), it is left unchanged.
    ///
    /// ## Supported JSON Value Types:
    /// - Strings
    /// - Numbers
    /// - Booleans
    /// - Arrays
    /// - Objects
    /// - Null
    ///
    /// ## Example:
    /// ```
    /// use serde_json::json;
    /// use json_deref::JsonResolvableFunctions;
    ///
    /// let input = json!({
    ///     "string_field": "Static Value",
    ///     "number_field": 123,
    ///     "boolean_field": true,
    ///     "null_field": null,
    ///     "absolute_field": "{/string_field}", // Absolute path
    ///     "relative_field": "{string_field}", // Sibling reference
    ///     "embedded_field": "Embedded: {string_field}", // Embedded dependency
    ///     "object_field": {
    ///         "parent_field": "{../relative_field}", // Resolves a field from parent object. Note that the dependencies will not be resolved recursively.
    ///         "out_of_bounds_field": "{../../boolean_field}" // Also resolves a field from parent object
    ///     },
    ///     "array_field": [
    ///         "Static Value", // Static value
    ///         "{../number_field}" // Sibling reference in array
    ///     ],
    ///     "unresolvable_field": "{/nonexistent}" // Unresolvable absolute path
    /// });
    ///
    /// let resolved = input.resolve_internal_dependencies();
    ///
    /// assert_eq!(resolved, json!({
    ///     "string_field": "Static Value", // Original value preserved
    ///     "number_field": 123, // Original value preserved
    ///     "boolean_field": true, // Original boolean preserved
    ///     "null_field": null, // Null value preserved
    ///     "absolute_field": "Static Value", // Resolved using absolute path
    ///     "relative_field": "Static Value", // Resolved sibling reference
    ///     "embedded_field": "Embedded: Static Value", // Embedded dependency resolved
    ///     "object_field": {
    ///         "parent_field": "{/string_field}", // Resolves a field from parent object. Note that the dependencies will not be resolved recursively.
    ///         "out_of_bounds_field": true // Also resolves a field from parent object
    ///     },
    ///     "array_field": [
    ///         "Static Value", // Static value unchanged
    ///         123 // Resolved sibling reference
    ///     ],
    ///     "unresolvable_field": "{/nonexistent}" // Unresolved dependency remains unchanged
    /// }));
    /// ```
    fn resolve_internal_dependencies(&self) -> Value {
        resolve_json(self)
    }

    /// Resolves the JSON as a template, using another JSON as the source for placeholders.
    ///
    /// The current JSON object acts as a template, and placeholders (e.g., `{/path/to/value}`) are
    /// resolved using the provided source JSON. All placeholders are replaced with their resolved
    /// values if they exist in the source JSON. Invalid or unresolvable placeholders remain unchanged.
    ///
    /// # Supported JSON Types:
    /// - Strings
    /// - Numbers
    /// - Booleans
    /// - Arrays
    /// - Objects
    /// - Null
    ///
    /// # Example:
    /// ```
    /// use serde_json::json;
    /// use json_deref::JsonResolvableFunctions;
    ///
    /// let template = json!({
    ///     "string_field": "{/data/string}",
    ///     "number_field": "{/data/number}",
    ///     "boolean_field": "{/data/boolean}",
    ///     "array_field": "{/data/array}",
    ///     "object_field": "{/data/object}",
    ///     "null_field": "{/data/null}",
    ///     "embedded_field": "Referenced: {/data/string}", // Embedded dependency
    ///     "unresolvable_field": "{/data/nonexistent}" // Unresolvable path
    /// });
    ///
    /// let source = json!({
    ///     "data": {
    ///         "string": "String Value",
    ///         "number": 123,
    ///         "boolean": false,
    ///         "array": ["a", "b", "c"],
    ///         "object": {"key": "value"},
    ///         "null": null
    ///     }
    /// });
    ///
    /// let resolved = template.resolve_template_with_source(&source);
    ///
    /// assert_eq!(resolved, json!({
    ///     "string_field": "String Value",
    ///     "number_field": 123,
    ///     "boolean_field": false,
    ///     "array_field": ["a", "b", "c"],
    ///     "object_field": {"key": "value"},
    ///     "null_field": null,
    ///     "embedded_field": "Referenced: String Value",
    ///     "unresolvable_field": "{/data/nonexistent}" // Paths not found in source remain unchanged
    /// }));
    /// ```
    fn resolve_template_with_source(&self, source: &Value) -> Value {
        resolve_template_with_source(self, source)
    }
}

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
