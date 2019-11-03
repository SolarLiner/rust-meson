use crate::utils::LRange;

#[derive(Clone, Debug, PartialEq)]
pub struct NodeData<T, L> {
    pub data: T,
    // pub subdir: String,
    pub range: LRange<L>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Node<L> {
    Identifier(NodeData<String, L>),
    String(NodeData<String, L>),
    Number(NodeData<f64, L>),
    Arglist(NodeData<Vec<Node<L>>, L>),
    Array(NodeData<Box<Node<L>>, L>),
    KeyValue(NodeData<(Box<Node<L>>, Box<Node<L>>), L>),
    KwargList(NodeData<Vec<Node<L>>, L>),
    Dict(NodeData<Box<Node<L>>, L>),
    Function(NodeData<Box<(Node<L>, Option<Node<L>>, Option<Node<L>>)>, L>),
}

peg::parser! {
    grammar meson() for str {
        use super::Node;
        use std::i32;
        rule eol() -> char = quiet!{"\r"? "\n"} {'\n'} / expected!("<EOL>")
        rule tab() -> char = quiet!{"\t"} {'\t'} / expected!("<TAB>")
        rule ws() = quiet!{([' ' | '\t'] / eol())*}

        pub rule identifier() -> Node<usize>
            = quiet!{ws() start:position!() i:$(['a'..='z' | 'A'..='Z' | '_'] ['a'..='z' | 'A'..='Z' | '0'..='9' | '_']*) end:position!() ws() {
            Node::Identifier(NodeData {data: i.to_owned(), range: LRange {start, end}})
        }} / expected!("identifier")

        rule char_special() -> char = "\\t" {'\t'} / "\\n" {'\n'} / r"\\" {'\\'}
        rule string_char() -> char = c:char_special() {c} / !("\\" / "'" / eol()) c:$([_]) {c.chars().next().unwrap()}
        rule string_inline() -> String
            = quiet!{ws() "'" s:string_char()* "'" ws() {s.into_iter().collect()}}
            / expected!("string")
        rule string_multiline() -> String
            = quiet!{ws() "'''" s:(string_char() / eol())* "'''" ws() {s.into_iter().collect()}}
            / expected!("multiline string")
        pub rule string() -> Node<usize>
            = start:position!() data:(string_inline() / string_multiline()) end:position!() {
            Node::String(NodeData {data, range: LRange {start, end}})
        }

        rule oct() -> i32 = n:$("-")? "0o" i:$(['0'..='7']+) {? i32::from_str_radix(i, 8).map_err(|_| "Couldn't parse octal") }
        rule dec() -> i32 = i:$("-"? ['0'..='9']+) {? i.parse::<i32>().map_err(|_| "Couldn't parse int")}
        rule hex() -> i32 = n:$("-")? "0x" i:$(['0'..='9' | 'a'..='f' | 'A'..='F']+) {? i.parse::<i32>().map_err(|_| "Couldn't parse hexadecimal")}
        rule float() -> f64 = f:$("-"? ['0'..='9']+ "." !"." ['0'..='9']+) {? f.parse::<f64>().map_err(|_| "Couldn't parse float")}
        pub rule number() -> Node<usize>
            = quiet!{start:position!() data:float() end:position!() {Node::Number(NodeData {data, range: LRange {start, end}})}}
             / expected!("integer")
             / quiet!{start:position!() i:(oct() / hex() / dec()) end:position!() {Node::Number(NodeData {data: i as f64, range: LRange {start, end}})}}
             / expected!("float")

        pub rule arglist() -> Node<usize>
            = quiet!{start:position!() data:value() ** (ws() "," ws()) end:position!() {
                Node::Arglist(NodeData {data, range: LRange {start, end}})
            }}
            / expected!("list inner")
        pub rule array() -> Node<usize>
            = quiet!{ws() start:position!() "[" l:arglist() "]" end:position!() ws() {Node::Array(NodeData {data: Box::new(l), range: LRange {start, end}})}}
            / expected!("array")

        pub rule keyvalue() -> Node<usize> = quiet!{ws() start:position!() k:identifier() ":" v:value() end:position!() ws() {
            Node::KeyValue(NodeData {data: (Box::new(k), Box::new(v)), range: LRange {start, end}})
        }} / expected!("key-value pair")

        pub rule kwlist() -> Node<usize> = quiet!{ws() start:position!() kv:keyvalue() ** (ws() "," ws()) end:position!() ws() {
            Node::KwargList(NodeData {data: kv, range: LRange {start, end}})
        }} / expected!("key-value list")

        pub rule dict() -> Node<usize> = quiet!{ws() start:position!() "{" k:kwlist() "}" end:position!() ws() {
            Node::Dict(NodeData {data: Box::new(k), range: LRange {start, end}})
        }} / expected!("dictionary")

        pub rule value() -> Node<usize> = v:(string() / number() / identifier() / array() / dict()) {v}

        pub rule function() -> Node<usize> = ws() start:position!() i:identifier() "(" a:arglist()? ","? k:kwlist()? ")" end:position!() ws() {
            Node::Function(NodeData {data: Box::new((i, a, k)), range: LRange {start, end}})
        }
    }
}

#[cfg(test)]
mod tests {
    use super::meson;
    use super::Node;
    use super::NodeData;
    use crate::utils::LRange;

    #[test]
    fn parse_ident() {
        let input = "ident";
        let output = Node::Identifier(NodeData {
            data: "ident".to_owned(),
            range: LRange { start: 0, end: 5 },
        });

        assert_eq!(Ok(output), meson::identifier(input));
        assert!(meson::identifier("32abc").is_err());
    }

    #[test]
    fn parse_string() {
        let input = "'hello'";
        let output = Node::String(NodeData {
            data: "hello".into(),
            range: LRange { start: 0, end: 7 },
        });
        assert_eq!(Ok(output), meson::string(input));
        assert!(meson::string("\"hello\"").is_err());
    }

    #[test]
    fn parse_number() {
        let input = "0o644";
        let output = Node::Number(NodeData {
            data: 0o644 as f64,
            range: LRange { start: 0, end: 5 },
        });

        assert_eq!(Ok(output), meson::number(input));
    }

    #[test]
    fn parse_arglist() {
        let input = "'hello', 1, abc";
        let output = Node::Arglist(NodeData {
            data: vec![
                Node::String(NodeData {
                    data: "hello".to_owned(),
                    range: LRange { start: 0, end: 7 },
                }),
                Node::Number(NodeData {
                    data: 1.0,
                    range: LRange { start: 9, end: 10 },
                }),
                Node::Identifier(NodeData {
                    data: "abc".to_owned(),
                    range: LRange { start: 12, end: 15 },
                }),
            ],
            range: LRange { start: 0, end: 15 },
        });

        assert_eq!(Ok(output), meson::arglist(input));
    }

    #[test]
    fn parse_function() {
        let input = "func(a, b: 'c')";
        let output = Node::Function(NodeData {
            data: Box::new((
                Node::Identifier(NodeData {
                    data: "func".to_owned(),
                    range:LRange {start: 0, end: 4}
                }),
                Node::Arglist(NodeData {
                    data: vec![
                        Node::Identifier(NodeData {
                            data: "a".to_owned(),
                            range: LRange {start: 5, end: 6}
                        })
                    ],
                    range: LRange {start: 5, end: 6}
                }),
                Node::KwargList(NodeData {
                    data:
                })
            ))
        })
    }
}
