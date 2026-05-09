use serde_json::{Result, Value};
use std::collections::HashMap;
use std::io;

fn lottie_compiler() {}

fn main() -> Result<()> {
    // let mut map = HashMap::new();

    let data = r#"{
      "v": "5.5.7",
      "fr": 30,
      "ip": 0,
      "op": 90,
      "w": 500,
      "h": 500,
      "nm": "Example",
      "ddd": 0,
      "assets": [],
      "layers": [
        {
          "ddd": 0,
          "ind": 1,
          "ty": 4,
          "nm": "Circle",
          "ks": {  },
          "shapes": [ ]
        }
      ]
    }"#;

    let my_map: HashMap<String, serde_json::Value> = serde_json::from_str(data)?;

    println!("{:?}", my_map.get("layers").expect("layers does not exist"));
    Ok(())
    // map.insert("name", "John");
    // map.insert("city", "New York");

    // // Convert HashMap to a JSON string
    // let json_string = serde_json::to_string(&map).unwrap();

    // println!("{}", json_string);
    // let mut input = String::new(); // Create an empty mutable String

    // io::stdin()
    //     .read_line(&mut input) // Read the input and store it in 'input'
    //     .expect("Failed to read line"); // Error handling
    // println!("hello, {}", input.trim());
}
