#![feature(debug_closure_helpers)]

use crate::model::Model;

mod parse;
mod model;

fn main() {
    let text = std::fs::read_to_string("resources/false.txt").unwrap();
    let parsed_context = parse::parse(&text);
    let model = Model::build(parsed_context);
    model.check();
}
