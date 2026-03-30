#![allow(unused)]

mod parse;
// mod model;

fn main() {
    let text = std::fs::read_to_string("resources/false.txt").unwrap();
    let parsed_context = parse::parse(&text);
    println!("{parsed_context:#?}");
    // let model = Model::build(parsed_context);
    // _ = model;
}
