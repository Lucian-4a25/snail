#[macro_use]
extern crate lazy_static;

use std::string::String;
use std::{env, fs};
mod ast;
mod file;
mod global;
mod parser;
mod statement;
mod tokenizer;

use statement::parse_top_level;

fn main() -> Result<(), file::ReadFileError> {
    env::set_var("RUST_BACKTRACE", "1");
    let file_path = file::get_file_path()?;
    let result: String = file::read_file_content(&file_path)?;
    println!("The content of file: ");
    println!("{}", result);
    println!();
    println!("----------------");
    println!();
    let mut parser = parser::Parser::new(result);
    parser.source_file = Some(file_path);
    let root = parse_top_level(&mut parser);
    // Convert the Ast Data structure to a JSON string.
    let serialized = serde_json::to_string_pretty(&root).unwrap();
    fs::write("output/example.json", serialized).unwrap();

    Ok(())
}
