use std::env;

mod encoder;
fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("Correct Usage: cargo run <location>");
    }
    //encoder::encode_dir(args.get(1).unwrap());
    encoder::verify(args.get(1).unwrap(), &String::from("src/sample.json"));
}
