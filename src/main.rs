// #![feature(debug_closure_helpers)]
#![feature(pattern)]

use crate::check::check_function_context;
use crate::parse::parse_file_as_function_context;
use crate::parse::Source;

mod check;
#[allow(unused)]
mod parse;

fn main() {
    let path = "resources/false.txt";
    let text = std::fs::read_to_string(path).unwrap();
    let source = Source::new(path, text.as_str());
    match parse_file_as_function_context(&source) {
        Ok(context) => {
            println!("parsing successful");
            // println!("{:#?}", context);

            check_function_context(context).unwrap();

            println!("checking successful");
        }
        Err(error) => {
            error
                .report()
                .eprint((path.to_owned(), &source.inner))
                .unwrap();
        }
    }
}
