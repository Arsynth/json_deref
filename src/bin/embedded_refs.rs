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
