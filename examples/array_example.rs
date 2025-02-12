use json_deref::resolve_json;
use serde_json::json;

fn main() {
    let input = json!({
        "users": [
            "{/user1}",
            "{/user2}"
        ],
        "user1": "Alice",
        "user2": "Bob"
    });

    let resolved_json = resolve_json(&input);

    println!("{}", input);
    println!("{}", resolved_json);
}
