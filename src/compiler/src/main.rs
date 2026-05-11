use gemini_rust::Gemini;
use serde_json::{Result as serde_Result, Value as serde_Value};
use std::env;
use std::{collections::HashMap, fs};

async fn script_analyzer() -> Result<(), Box<dyn std::error::Error>> {
    // analyzes script to find moments for visuals (lotties)
    // {

    let api_key = env::var("GEMINI_API_KEY")?;
    let client = Gemini::new(api_key)?;

    let response = client
        .generate_content()
        .with_system_prompt("You reply only in Rhymes")
        .with_user_message("Explain quantum computing in one sentence.")
        .execute()
        .await?;

    println!("{}", response.text());

    // let file_path = "../../public/data.json";
    // let contents = fs::read_to_string(file_path).expect("failed to read file");

    // println!("contents {contents}");

    Ok(())
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

    // todo!()
}

fn animation_spec() {
    // gemini designs each animation based on a constrained vocabulary syntax
    todo!()
}

fn lottie_compiler() {
    // maps keywords to valid lottie json
    todo!()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    script_analyzer().await?;
    // let data = r#"{
    //   "v": "5.5.7",
    //   "fr": 30,
    //   "ip": 0,
    //   "op": 90,
    //   "w": 500,
    //   "h": 500,
    //   "nm": "Example",
    //   "ddd": 0,
    //   "assets": [],
    //   "layers": [
    //     {
    //       "ddd": 0,
    //       "ind": 1,
    //       "ty": 4,
    //       "nm": "Circle",
    //       "ks": {  },
    //       "shapes": [ ]
    //     }
    //   ]
    // }"#;

    // let my_map: HashMap<String, serde_json::Value> = serde_json::from_str(data)?;

    // println!("{:?}", my_map.get("layers").expect("layers does not exist"));
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
