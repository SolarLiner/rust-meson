use crate::utils::LRange;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeData<T, L> {
    pub data: T,
    // pub subdir: String,
    pub range: LRange<L>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Operation {
    And,
    Or,
    Not,
    Equals,
    NotEquals,
    LessThan,
    LessEqual,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Node<L> {
    Identifier(NodeData<String, L>),
    String(NodeData<String, L>),
    Number(NodeData<f64, L>),
    Arglist(NodeData<Vec<Node<L>>, L>),
    Array(NodeData<Box<Node<L>>, L>),
    KeyValue(NodeData<Box<(Node<L>, Node<L>)>, L>),
    KwargList(NodeData<Vec<Node<L>>, L>),
    Dict(NodeData<Box<Node<L>>, L>),
    Assignment(NodeData<Box<(Node<L>, Node<L>)>, L>),
    PlusAssignment(NodeData<Box<(Node<L>, Node<L>)>, L>),
    Addition(NodeData<Vec<Node<L>>, L>),
    MemberAccess(NodeData<Vec<Node<L>>, L>),
    Function(NodeData<Box<(Node<L>, Node<L>, Node<L>)>, L>),
    Binop(NodeData<(Operation, Box<(Node<L>, Node<L>)>), L>),
    Unop(NodeData<(Operation, Box<Node<L>>), L>),
    IfBlock(NodeData<Box<(Node<L>, Node<L>, Node<L>)>, L>),
    ForeachBlock(NodeData<Box<(Node<L>, Node<L>)>, L>),
    CodeBlock(NodeData<Vec<Node<L>>, L>),
    Empty,
}

impl Node<usize> {
    pub fn transform_offset<'a>(self, input: &'a str) -> Node<LRange<usize>> {}
}

peg::parser! {
    grammar meson() for str {
        use super::Node;
        use std::i32;
        rule eol() -> char = quiet!{"\r"? "\n"} {'\n'} / expected!("<EOL>")
        rule tab() -> char = quiet!{"\t"} {'\t'} / expected!("<TAB>")
        rule ws() = quiet!{([' ' | '\t'] / eol())*}

        #[cache]
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

        rule oct() -> i32 = n:("-")? "0o" i:$(['0'..='7']+) {?
            let parsed = i32::from_str_radix(i, 8).map_err(|_| "Couldn't parse octal");
            if n.is_some() {
                parsed.map(|v| v * -1)
            } else {
                parsed
            }
        }
        rule dec() -> i32 = i:$("-"? ['0'..='9']+) {? i.parse::<i32>().map_err(|_| "Couldn't parse int")}
        rule hex() -> i32 = n:("-")? "0x" i:$(['0'..='9' | 'a'..='f' | 'A'..='F']+) {?
            let parsed = i32::from_str_radix(i, 16).map_err(|_| "Couldn't parse hexadecimal");
            if n.is_some() {
                parsed.map(|v| v * -1)
            } else {
                parsed
            }
        }
        rule float() -> f64 = f:$("-"? ['0'..='9']+ "." !"." ['0'..='9']+) {? f.parse::<f64>().map_err(|_| "Couldn't parse float")}
        pub rule number() -> Node<usize>
            = quiet!{start:position!() data:float() end:position!() {Node::Number(NodeData {data, range: LRange {start, end}})}}
             / quiet!{start:position!() i:(oct() / hex() / dec()) end:position!() {Node::Number(NodeData {data: i as f64, range: LRange {start, end}})}}
             / expected!("number")

        pub rule arglist() -> Node<usize>
            = quiet!{start:position!() data:(v:value() !":" {v}) ** (ws() "," ws()) end:position!() {
                Node::Arglist(NodeData {data, range: LRange {start, end}})
            }}
            / expected!("list inner")
        pub rule array() -> Node<usize>
            = quiet!{ws() start:position!() "[" l:arglist() "]" end:position!() ws() {Node::Array(NodeData {data: Box::new(l), range: LRange {start, end}})}}
            / expected!("array")

        pub rule keyvalue() -> Node<usize> = quiet!{ws() start:position!() k:identifier() ":" v:value() end:position!() ws() {
            Node::KeyValue(NodeData {data: Box::new((k, v)), range: LRange {start, end}})
        }} / expected!("key-value pair")

        pub rule kwlist() -> Node<usize> = quiet!{ws() start:position!() kv:keyvalue() ** (ws() "," ws()) end:position!() ws() {
            Node::KwargList(NodeData {data: kv, range: LRange {start, end}})
        }} / expected!("key-value list")

        pub rule dict() -> Node<usize> = quiet!{ws() start:position!() "{" k:kwlist() "}" end:position!() ws() {
            Node::Dict(NodeData {data: Box::new(k), range: LRange {start, end}})
        }} / expected!("dictionary")

        pub rule add() -> Node<usize> = ws() start:position!() a:value_raw() **<2,> (ws() "+" ws()) end:position!() ws() {
            Node::Addition(NodeData {
                data: a,
                range: LRange {start, end}
            })
        }

        pub rule value_raw() -> Node<usize> = v:(string() / function() / member_access() / identifier() / array() / dict() / number()) {v}
        pub rule value() -> Node<usize> = v:(add() / value_raw())

        pub rule function() -> Node<usize>
            = quiet!{ws() start:position!() i:(member_access() / identifier()) "(" a:arglist() k:("," k:kwlist() {k})? ")" end:position!() ws() {
            Node::Function(NodeData {data: Box::new((i, a, k.unwrap_or(Node::Empty))), range: LRange {start, end}})
        }} / expected!("function call")

        #[cache]
        pub rule member_access() -> Node<usize> = ws() start:position!() m:(identifier()) **<2,> (ws() "." ws()) end:position!() ws() {
            Node::MemberAccess(NodeData {
                data: m,
                range: LRange {start, end}
            })
        }

        pub rule assignment() -> Node<usize>
            = quiet!{ws() start:position!() m:(member_access() / identifier()) ws() "=" ws() a:value() end:position!() ws() {
            Node::Assignment(NodeData {
                data: Box::new((m, a)),
                range: LRange {start, end}
            })
        }} / expected!("assignment")

        pub rule plus_assignment() -> Node<usize>
            = quiet!{ws() start:position!() m:(member_access() / identifier()) ws() "+=" ws() a:(value()) end:position!() ws() {
            Node::PlusAssignment(NodeData {
                data: Box::new((m, a)),
                range: LRange {start, end}
            })
        }} / expected!("plus assignment")

        pub rule instruction() -> Node<usize>
            = ws() n:(function() / assignment()) ws() {n}

        pub rule code_block() -> Node<usize>
            = ws() start:position!() i:(instruction()) ** (ws() eol() ws()) end:position!() ws() {
            Node::CodeBlock(NodeData {
                data: i,
                range: LRange {start, end}
            })
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
        let input = "-0o644";
        let output = Node::Number(NodeData {
            data: -0o644 as f64,
            range: LRange { start: 0, end: 6 },
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
                    range: LRange { start: 0, end: 4 },
                }),
                (Node::Arglist(NodeData {
                    data: vec![Node::Identifier(NodeData {
                        data: "a".to_owned(),
                        range: LRange { start: 5, end: 6 },
                    })],
                    range: LRange { start: 5, end: 6 },
                })),
                (Node::KwargList(NodeData {
                    data: vec![Node::KeyValue(NodeData {
                        data: Box::new((
                            Node::Identifier(NodeData {
                                data: "b".to_owned(),
                                range: LRange { start: 8, end: 9 },
                            }),
                            Node::String(NodeData {
                                data: "c".to_owned(),
                                range: LRange { start: 10, end: 14 },
                            }),
                        )),
                        range: LRange { start: 8, end: 14 },
                    })],
                    range: LRange { start: 8, end: 14 },
                })),
            )),
            range: LRange { start: 0, end: 15 },
        });

        assert_eq!(Ok(output), meson::function(input));
    }

    #[test]
    fn parse_assignment() {
        let input = "a = 'hello'";
        let output = Node::Assignment(NodeData {
            data: Box::new((
                Node::Identifier(NodeData {
                    data: "a".to_owned(),
                    range: LRange { start: 0, end: 1 },
                }),
                Node::String(NodeData {
                    data: "hello".to_owned(),
                    range: LRange { start: 4, end: 11 },
                }),
            )),
            range: LRange { start: 0, end: 11 },
        });
        assert_eq!(Ok(output), meson::assignment(input));
    }

    #[test]
    fn parse_plus_assignment() {
        let input = "a += 1.2";
        let output = Node::PlusAssignment(NodeData {
            data: Box::new((
                Node::Identifier(NodeData {
                    data: "a".to_owned(),
                    range: LRange { start: 0, end: 1 },
                }),
                Node::Number(NodeData {
                    data: 1.2,
                    range: LRange { start: 5, end: 8 },
                }),
            )),
            range: LRange { start: 0, end: 8 },
        });
        assert_eq!(Ok(output), meson::plus_assignment(input));
    }

    #[test]
    fn parse_addition() {
        let input = "'hello ' + world+'!'";
        let output = Node::Addition(NodeData {
            data: vec![
                Node::String(NodeData {
                    data: "hello ".to_owned(),
                    range: LRange { start: 0, end: 9 },
                }),
                Node::Identifier(NodeData {
                    data: "world".to_owned(),
                    range: LRange { start: 11, end: 16 },
                }),
                Node::String(NodeData {
                    data: "!".to_owned(),
                    range: LRange { start: 17, end: 20 },
                }),
            ],
            range: LRange { start: 0, end: 20 },
        });
        assert_eq!(Ok(output), meson::add(input));
    }

    #[test]
    fn parse_member_access() {
        let input = "meson.project_name";
        let output = Node::MemberAccess(NodeData {
            data: vec![
                Node::Identifier(NodeData {
                    data: "meson".to_owned(),
                    range: LRange { start: 0, end: 5 },
                }),
                Node::Identifier(NodeData {
                    data: "project_name".to_owned(),
                    range: LRange { start: 6, end: 18 },
                }),
            ],
            range: LRange { start: 0, end: 18 },
        });
        assert_eq!(Ok(output), meson::member_access(input));
    }
}
