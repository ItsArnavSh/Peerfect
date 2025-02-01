use std::env;
#[warn(special_module_name)]
mod lib;
fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("Correct Usage: cargo run <location>");
    }
    //encoder::encode_dir(args.get(1).unwrap());
    lib::verify(args.get(1).unwrap(), &String::from("src/sample.json"));
}
