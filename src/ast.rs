use crate::utils::LRange;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct NodeData<T> {
    pub data: T,
    pub subdir: String,
    pub range: LRange,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Node<'a> {}

peg::parser! {
    grammar utils() for str {
        rule eol() -> char = quiet!{"\r"? "\n"} {'\n'} / expected!("<EOL>")
        rule tab() -> char = quiet!{"\t"} {'\t'} / expected!("<TAB>")
        rule ws() = quiet!{([' ' | '\t'] / eol())*}

        rule char_special() -> char = "\\t" {'\t'} / "\\n" {'\n'} / r"\\" {'\\'}
        rule string_char() -> char = c:char_special() {c} / !("\\" / "'" / eol()) c:$([_]) {c.chars().next().unwrap()}
        rule string_inline() -> String = ws() "'" s:string_char()* "'" ws() {s.into_iter().collect()}
        rule string_multiline() -> String = ws() "'''" s:(string_char() / eol())* "'''" {s.into_iter().collect()}
        rule string() -> String = s:(string_inline() / string_multiline()) {s}
    }
}
