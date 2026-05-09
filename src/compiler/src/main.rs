fn main() {
    println!("Hello, world!");
    println!("this is Bedrock's Compiler!");
    random_number();
}


fn random_number() -> f32 {
    let random: f32 = rand::random_range(0.0..=1e9);
    println!("{}", random);
    random
}