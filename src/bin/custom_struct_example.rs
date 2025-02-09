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