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

    println!("{}", input);
    println!("{}", resolved_json);

}