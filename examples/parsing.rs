use meson::parse::parse;

fn main() {
    let ast = parse("true or false");
    println!("{:?}", ast);
}
