use meson::ast;
use std::io::{self, Read};

fn main() -> Result<(), String> {
	let mut input_str = String::new();
	let mut input_buf = io::stdin();
	input_buf.read_to_string(&mut input_str).map_err(|e| format!("{:?}", e))?;

	match ast::parse(&input_str) {
		Ok(ast) => Ok(println!("{:#?}", ast)),
		Err(e) => Err(format!("{:?}", e)),
	}
}
