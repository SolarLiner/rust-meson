use meson::ast;
use std::io::{self, Read};

fn main() -> io::Result<()> {
	let mut input_str = String::new();
	let mut input_buf = io::stdin();
	input_buf.read_to_string(&mut input_str)?;

	match ast::parse(&input_str) {
		Ok(ast) => Ok(println!("{:#?}", ast)),
		Err(e) => {
			eprintln!("{}", e);
			Err(io::Error::from(io::ErrorKind::Other))
		}
	}
}
