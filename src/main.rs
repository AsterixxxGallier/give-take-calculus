#![feature(debug_closure_helpers)]
#![feature(pattern)]

use crate::parse2::Source;

#[allow(unused)]
mod parse;
mod parse2;
#[allow(unused)]
mod model;

fn main() {
    let path = "resources/false.txt";
    let text = std::fs::read_to_string(path).unwrap();
    let source = Source::new(path, text.as_str());
    match parse2::parse_file(&source) {
        Ok(context) => {
            println!("parsing successful");
            println!("{:#?}", context);
        }
        Err(error) => {
            error.report().eprint((path.to_owned(), &source.inner)).unwrap();
        }
    }
}
