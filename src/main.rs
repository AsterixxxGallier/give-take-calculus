#![feature(debug_closure_helpers)]
#![feature(pattern)]

use crate::model::Model;
use crate::parse::Source;

mod parse;
mod model;

fn main() {
    let path = "resources/false.txt";
    let text = std::fs::read_to_string(path).unwrap();
    let source = Source::new(path, text.as_str());
    match parse::parse_file(&source) {
        Ok(context) => {
            println!("parsing successful");
            // println!("{:#?}", context);

            let model = Model::build(context);
            model.check();
        }
        Err(error) => {
            error.report().eprint((path.to_owned(), &source.inner)).unwrap();
        }
    }
}
