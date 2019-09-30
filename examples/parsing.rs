use mparser::parse::Token;

fn main() {
    let tok_vec: Vec<_> = Token::parse_input("project('test', 'c')\nexecutable('test', 'main.c')").into_iter().collect();
    println!("{:?}", tok_vec);
}