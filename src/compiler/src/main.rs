use serde_json::{Result, Value};
use std::collections::HashMap;

fn script_analyzer() {
    // analyzes script to find moments for visuals (lotties)
    // {
    // use rust reqwest to fetch data.json file
    // 
    // use gemini api to analyze script to find moments that should be visuals
    // 
    // returns json only, no commentary
    // 
    // example: 
    // 
    // [
    //   {
    //     "index": 0,
    //     "concept": "growth",
    //     "mood": "energetic",
    //     "metaphor": "expanding circle",
    //     "duration": 24
    //   }
    // ]

    // }

    todo!()
}

fn animation_spec() {
    // gemini designs each animation based on a constrained vocabulary syntax
    todo!()
}

fn lottie_compiler() {
    // maps keywords to valid lottie json
    todo!()
}

fn main() -> Result<()> {
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
