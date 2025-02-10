use json_deref::resolve_json_reader_to_object;
use serde::Deserialize;
use std::fs::File;

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
    let file = r#"
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
    "#.as_bytes();

    let response: Response = resolve_json_reader_to_object(file).unwrap();

    println!("{:?}", response);
}
