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

println!("{}", input);
println!("{}", resolved_json);

}