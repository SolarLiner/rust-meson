use std::ops::Range;
use crate::utils::{Location, LRange};
use logos::{Lexer, Logos};

#[derive(Clone, Copy, Debug, Logos, PartialEq)]
pub enum RawToken {
    #[end]
    End,
    #[error]
    Error,
    #[regex = r"[a-zA-Z_][a-zA-Z0-9_]*"]
    Ident,
    #[regex = r"0[bB][01]+|0[oO][0-7]+|0[xX][0-9a-fA-F]+|[0-9]+"]
    Number,
    #[token = r"\n"]
    EolCont,
    #[token = "\n"]
    Eol,
    #[regex = r"'''(.|\n)*'''"]
    MultilineString,
    #[regex = r"#.*"]
    Comment,
    #[token = "("]
    LeftParent,
    #[token = ")"]
    RightParent,
    #[token = "["]
    LeftBracket,
    #[token = "]"]
    RightBracket,
    #[token = "{"]
    LeftBrace,
    #[token = "}"]
    RightBrace,
    #[token = "\""]
    DoubleQuote,
    #[regex = r"'([^'\\]|(\\.))*'"]
    String,
    #[token = ","]
    Comma,
    #[token = "+="]
    PlusAssign,
    #[token = "."]
    Dot,
    #[token = "+"]
    Plus,
    #[token = "-"]
    Dash,
    #[token = "*"]
    Star,
    #[token = "%"]
    Percent,
    #[token = "/"]
    FSlash,
    #[token = ":"]
    Colon,
    #[token = "=="]
    Equal,
    #[token = "!="]
    NEqual,
    #[token = "="]
    Assign,
    #[token = "<="]
    LessEqual,
    #[token = "<"]
    LessThan,
    #[token = ">="]
    GreaterEqual,
    #[token = ">"]
    GreaterThan,
    #[token = "?"]
    QuestionMark,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Token<'a> {
    Ident(TokenData<&'a str>),
    Number(TokenData<f64>),
    String(TokenData<(&'a str, bool)>),
    Comment(TokenData<&'a str>),
    Punct(TokenData<&'a str>),
    Error,
    End,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TokenData<T> {
    token: RawToken,
    data: T,
    range: LRange,
}

pub struct TokenIter<'a>(Lexer<RawToken, &'a str>);

impl<'a, T> TokenData<T> {
    pub fn new(s: &'a str, token: RawToken, data: T, range: Range<usize>) -> Self {
        Self {
            token,
            data,
            range: LRange::from_offset(s, range.start, range.end),
        }
    }
}

impl<'a> Token<'a> {
    pub fn parse_input(input: &'a str) -> TokenIter {
        TokenIter::new(input)
    }

    pub fn map_raw_token(input: &'a str, token: RawToken, range: Range<usize>) -> Self {
        match token {
            RawToken::Ident => Token::Ident(TokenData::new(input, token, &input[range.clone()], range)),
            RawToken::Number => {
                if let Ok(data) = (&input[range.clone()]).parse() {
                    Token::Number(TokenData::new(input, token, data, range))
                } else {
                    Token::Error
                }
            }
            RawToken::MultilineString => {
                let mut inner_range = range.clone();
                inner_range.start += 3;
                inner_range.end -= 3;
                let data: &'a str = &input[inner_range];
                Token::String(TokenData::new(input, token, (data, true), range))
            }
            RawToken::String => {
                let mut inner_range = range.clone();
                inner_range.start += 1;
                inner_range.end -= 1;
                let data: &'a str = &input[inner_range];
                Token::String(TokenData::new(input, token, (data, false), range))
            }
            RawToken::Error => Token::Error,
            RawToken::End => Token::End,
            _ => Token::Punct(TokenData::new(input, token, &input[range.clone()], range))
        }
    }
}

impl<'a> TokenIter<'a> {
    pub fn new(input: &'a str) -> Self {
        let lexer = RawToken::lexer(input);
        Self(lexer)
    }
}

impl<'a> Iterator for TokenIter<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let res = Token::map_raw_token(self.0.source, self.0.token, self.0.range());
        self.0.advance();
        match res {
            Token::Error => None,
            Token::End => None,
            x => Some(x)
        }
    }
}
