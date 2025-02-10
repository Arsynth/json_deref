# json_deref

`json_deref` is a Rust library for resolving internal dependencies in JSON structures. It automatically replaces placeholders (e.g., `{some_dependency}`) in JSON strings with their corresponding values, simplifying the handling of self-referencing or cross-referencing JSON data. The library provides multiple utility functions for resolving JSON either into a `Value` (from `serde_json`) or into user-defined types.

## Key Features

- **Dependency Resolution**: Automatically resolves JSON placeholders (`{}`) to their corresponding values.
- **Absolute Path Expansion**: Converts relative dependency paths into absolute paths.
- **Resolving Embedded Dependencies** Resolves multiple embedded dependencies in text
- **Graceful Fallbacks** If a dependency cannot be resolved, the placeholder will be replaced by its absolute path.
- **No Recursion** Only top-level dependencies are resolved. Nested dependencies in resolved text are not recursively processed. This prevents infinite resolution.
- **Error-Tolerant** The library does not generate errors during resolution. All dependencies are processed in a fail-safe manner. If a dependency cannot be resolved, the library will replace it with its absolute path (e.g., /some/absolute/path) rather than throwing an error.

## Installation

Add `json_deref` to your `Cargo.toml` dependencies:

```toml
[dependencies]
json_deref = "0.1.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

Example Usage
Below are examples of how to use json_deref to resolve dependencies in JSON data.

## Resolving JSON to a serde_json::Value

```rust
use json_deref::resolve_json;
use serde_json::json;

fn main() {
    let input = json!({
        "name": "Alice",
        "profile": {
            "bio": "{/details/bio}",
            "age": "{/details/age}"
        },
        "details": {
            "bio": "Software Engineer",
            "age": 30
        }
    });

    let resolved_json = resolve_json(&input);
    println!("{}", resolved_json);
}
```

Output:

```json
{
    "name": "Alice",
    "profile": {
        "bio": "Software Engineer",
        "age": 30
    },
    "details": {
        "bio": "Software Engineer",
        "age": 30
    }
}
```

## Using Relative Placeholder References

The library supports resolving relative references within JSON data. Relative paths are computed based on the location of the placeholder.

```rust
use json_deref::resolve_json;
use serde_json::json;

fn main() {
    let input = json!({
        "project": {
            "name": "json_deref",
            "author": "{../author/name}"
        },
        "author": {
            "name": "Arsynth"
        }
    });

    let resolved_json = resolve_json(&input);

    println!("{}", resolved_json);

}
```

Output:

```json
{
    "project": {
        "name": "json_deref",
        "author": "Arsynth"
    },
    "author": {
        "name": "Arsynth"
    }
}
```

## Resolving Embedded Dependencies in Text

Here’s an additional example demonstrating a JSON structure with a field containing a message that includes multiple dependencies (embedded within the text)

```rust
use json_deref::resolve_json;
use serde_json::json;

fn main() {
    let input = json!({
        "greeting": "Hello, {user_name}! You have {message_count} new messages.",
        "user_name": "Alice",
        "message_count": 5
    });

    let resolved_json = resolve_json(&input);

    println!("{}", resolved_json);
}
```

Output:

```json
{"greeting":"Hello, Alice! You have 5 new messages.","message_count":5,"user_name":"Alice"}
```

## Converting Resolved JSON into a Custom Struct

You can resolve JSON and deserialize it directly into a custom type using resolve_json_to_object.

```rust
use json_deref::resolve_json_to_object;
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct Response {
    profile: Profile,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct Profile {
    bio: String,
    age: String,
}

fn main() {
    let input = json!({
        "profile": {
            "bio": "{/details/bio}",
            "age": "{/details/age}"
        },
        "details": {
            "bio": "Software Engineer",
            "age": 30
        }
    });

    let response: Response = resolve_json_to_object(&input).expect("Failed to resolve JSON");

    println!("{:?}", response);

}
```

Output:

```rust
Response { profile: Profile { bio: "Software Engineer", age: "30" } }
```

## Resolving JSON from a Reader

You can resolve JSON directly from a reader (for example, a file or a network stream):

```rust
#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct Response {
    profile: Profile,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct Profile {
    bio: String,
    age: String,
}

fn main() -> Result<(), Box> {
    let file = File::open("example.json")?;
    let profile: Response = resolve_json_reader_to_object(file)?;

    println!("{:?}", profile);
    Ok(())

}
```

Examples of Supported Formats
Input with Object:

```json
{
    "profile": {
        "bio": "{/details/bio}",
        "age": "{/details/age}"
    },
    "details": {
        "bio": "Software Engineer",
        "age": 30
    }
}
```

Output:

```rust
Response { profile: Profile { bio: "Software Engineer", age: "30" } }
```

Input with Array:

```json
{
    "users": [
        "{/user1}",
        "{/user2}"
    ],
    "user1": "Alice",
    "user2": "Bob"
}
```

Resolved Output:

```json
{
    "users": [
        "Alice",
        "Bob"
    ],
    "user1": "Alice",
    "user2": "Bob"
}
```

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
