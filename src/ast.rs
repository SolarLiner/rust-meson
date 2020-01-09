use pest::Parser;
use pest::iterators::{Pairs, Pair};
use std::collections::HashMap;
use nom::lib::std::ops::DispatchFromDyn;

#[derive(Parser)]
#[grammar = "./meson.pest"]
struct MesonParser;

pub struct NodeValue<'a, T> {
    pub span: pest::Span<'a>,
    pub value: T,
}

pub type Identifier<'a> = NodeValue<'a, String>;
pub type MemberAccess<'a> = NodeValue<'a, Vec<Identifier<'a>>>;
pub type Arguments<'a> = NodeValue<'a, Vec<Value<'a>>>;
pub type KeywordArguments<'a> = HashMap<Identifier<'a>, Value<'a>>;

pub enum Value<'a> {
    Number(f64),
    String(String),
    Array(Vec<Value<'a>>),
    Dict(HashMap<Identifier<'a>, Value<'a>>),
    Function(MemberAccess<'a>, Arguments<'a>, KeywordArguments<'a>),
}

pub enum Unop {
    Not,
}

pub enum Binop {
    And,
    Or,
    GreaterThan,
    GreaterEqual,
    LessThan,
    LessEqual,
}

pub enum Expr<'a> {
    Value(NodeValue<'a, Value<'a>>),
    Unop(Unop, NodeValue<'a, Expr<'a>>),
    Binop(Binop, NodeValue<'a, (Expr<'a>, Expr<'a>)>),
}

pub enum Instruction<'a> {
    Conditional {
        condition: Expr<'a>,
        if_block: CodeBlock<'a>,

    },
    Value(Value<'a>),
    Assignment(MemberAccess<'a>, Value<'a>),
    PlusAssignment(MemberAccess<'a>, Value<'a>),
}

pub type CodeBlock<'a> = Vec<Instruction<'a>>;

pub fn parse(input: &str) -> Result<Pairs<Rule>, pest::error::Error<Rule>> {
    MesonParser::parse(Rule::program, input.as_ref())
}

fn code_block(pair: Pair<Rule>) -> CodeBlock {
    match pair.as_rule() {
        Rule::program => pair.into_inner().map(code_block).collect(),
        RUle::code_block => pair.into_inner().map(instruction).collect(),
        _ => unreachable!(),
    }
}

fn instruction(pair: Pair<Rule>) -> Instruction {
    match pair.as_rule() {
        Rule::assignment => {
            let macc = pair.into_inner().filter(|p| p.as_rule() == Rule::member_access).next().unwrap();
            let val = pair.into_inner().filter(|p| p.as_rule() == Rule::value).next().unwrap();
            Instruction::Assignment(member_access(macc), value(val))
        }
        Rule::expr => expr(pair),
    }
}

fn expr(pair: Pair<Rule>) -> Expr {
    match pair.as_rule() {
        Rule::expr {
            let value = statement(pair.into_inner().filter(|p| p.as_rule() == Rule::statement).next().unwrap());
            Expr::Unop(Unop::Not, NodeValue { span: pair.as_span(), value })
        },
        Rule::
    }
}

fn statement(pair: Pair<Rule>) -> Expr {
    match pair.as_rule() {
        Rule::expr => expr(pair),
        Rule::value => Expr::Value(NodeValue { span: pair.as_span(), value: value(pair) }),
        _ => unreachable!(),
    }
}

fn member_access(pair: Pair<Rule>) -> MemberAccess {
    unimplemented!()
}

fn value(pair: Pair<Rule>) -> Value {
    unimplemented!()
}
