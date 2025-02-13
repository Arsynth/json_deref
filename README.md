# json_deref

`json_deref` is a Rust library for resolving internal dependencies in JSON structures. It automatically replaces placeholders (e.g., `{some_dependency}`) in JSON strings with their corresponding values, simplifying the handling of self-referencing or cross-referencing JSON data. The library provides multiple utility functions for resolving JSON either into a `Value` (from `serde_json`) or into user-defined types.

## Key Features

- **Dependency resolution for self-referentional JSON** Automatically resolves JSON placeholders (like `{neighbour_value}`, `{../parent_value}`, e.t.c) to their corresponding values.
- **Dependency resolution for template JSON** Automatically resolves JSON placeholders in template (like `{/object/field}`) to their corresponding values using data source JSON.
- **Absolute Path Expansion**: Converts relative dependency paths into absolute paths.
- **Resolving Embedded Dependencies** Resolves multiple embedded dependencies in text
- **Graceful Fallbacks** If a dependency cannot be resolved, the placeholder will be replaced by its absolute path.
- **No Recursion** Only top-level dependencies are resolved. Nested dependencies in resolved text are not recursively processed. This prevents infinite resolution.
- **Error-Tolerant** The library does not generate errors during resolution. All dependencies are processed in a fail-safe manner. If a dependency cannot be resolved, the library will replace it with its absolute path (e.g., /some/absolute/path) rather than throwing an error.

## Installation

Add `json_deref` to your `Cargo.toml` dependencies:

```toml
[dependencies]
json_deref = "0.2.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

## Examples of functionality

- Resolving internal dependencies

```rust
let input = json!({
    "string_field": "Static Value",
    "number_field": 123,
    "boolean_field": true,
    "null_field": null,
    "absolute_field": "{/string_field}", // Absolute path
    "relative_field": "{string_field}", // Sibling reference
    "embedded_field": "Embedded: {string_field}", // Embedded dependency
    "object_field": {
        "parent_field": "{../relative_field}", // Resolves a field from parent object. Note that the dependencies will not be resolved recursively.
        "out_of_bounds_field": "{../../boolean_field}" // Also resolves a field from parent object
        },
    "array_field": [
        "Static Value", // Static value
        "{../number_field}" // Sibling reference in array
    ],
    "unresolvable_field": "{/nonexistent}" // Unresolvable absolute path
});

let resolved = input.resolve_internal_dependencies();
    assert_eq!(resolved, json!({
        "string_field": "Static Value", // Original value preserved
        "number_field": 123, // Original value preserved
        "boolean_field": true, // Original boolean preserved
        "null_field": null, // Null value preserved
        "absolute_field": "Static Value", // Resolved using absolute path
        "relative_field": "Static Value", // Resolved sibling reference
        "embedded_field": "Embedded: Static Value", // Embedded dependency resolved
        "object_field": {
            "parent_field": "{/string_field}", // Resolves a field from parent object. Note that the dependencies will not be resolved recursively.
            "out_of_bounds_field": true // Also resolves a field from parent object
        },
        "array_field": [
            "Static Value", // Static value unchanged
            123 // Resolved sibling reference
    ],
    "unresolvable_field": "{/nonexistent}" // Unresolved dependency remains unchanged
}));
```

- Resolving dependencies using both template and data source JSON

```rust
let template = json!({
        "string_field": "{/data/string}",
        "number_field": "{/data/number}",
        "boolean_field": "{/data/boolean}",
        "array_field": "{/data/array}",
        "object_field": "{/data/object}",
        "null_field": "{/data/null}",
        "embedded_field": "Referenced: {/data/string}", // Embedded dependency
        "unresolvable_field": "{/data/nonexistent}" // Unresolvable path
    });

    let source = json!({
        "data": {
            "string": "String Value",
            "number": 123,
            "boolean": false,
            "array": ["a", "b", "c"],
            "object": {"key": "value"},
            "null": null
        }
    });

    let resolved = template.resolve_template_with_source(&source);

    assert_eq!(resolved, json!({
        "string_field": "String Value",
        "number_field": 123,
        "boolean_field": false,
        "array_field": ["a", "b", "c"],
        "object_field": {"key": "value"},
        "null_field": null,
        "embedded_field": "Referenced: String Value",
        "unresolvable_field": "{/data/nonexistent}" // Paths not found in source remain unchanged
    }));
```

See additional examples in '/examples' dir

## How It Works

json_deref uses the following steps to resolve a JSON structure:

### Path Mapping

A map of dependencies is created by scanning the JSON strings for placeholders like {key} or {path/to/key}.

### Path Conversion

Relative paths are converted to absolute paths for easier reference resolution.

### Dependency Extraction

For each absolute path, the corresponding value in the JSON is extracted.

### Resolution

Placeholders in the JSON are replaced with their resolved values.

## Contributing

If you'd like to improve this library or suggest new features, feel free to fork the repository and submit a pull request.

Clone the repository:
   git clone <https://github.com/Arsynth/json_deref.git>
Create a feature branch:
   git checkout -b feature/my-new-feature
Submit your changes.
License
json_deref is licensed under the Apache-2.0 License.

Feel free to use, modify, and distribute the library in accordance with the terms of the license.
