#![allow(unused)]

mod parse;

fn main() {
    let text = std::fs::read_to_string("resources/false.txt").unwrap();
    let statements = parse::parse(&text);
    println!("{statements:#?}");
}
