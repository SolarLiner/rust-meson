use std::ops::Range;

use anyhow::{Context, Result as ResA};
use pest::error::Error;
use pest::iterators::{Pair, Pairs};
use pest::Parser;
use pest::prec_climber::{Assoc, Operator, PrecClimber};

use crate::ast::{AstExpr, BinopT};
use crate::utils::{Location, LRange};

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct RustParser;

lazy_static! {
    static ref PC_EXPR: PrecClimber<Rule> = {
        use Assoc::*;
        use Rule::*;
        PrecClimber::new(vec![
            Operator::new(binop_and, Left),
            Operator::new(binop_or, Left),
            Operator::new(binop_add, Left) | Operator::new(binop_sub, Left),
            Operator::new(binop_mul, Left) | Operator::new(binop_div, Left)
        ])
    };
}

pub fn parse(input: &str) -> ResA<AstExpr> {
    let pairs = parse_raw(input)?;
    into_expr(pairs)
}

fn parse_raw(input: &str) -> Result<Pairs<Rule>, Error<Rule>> {
    RustParser::parse(Rule::binop_expr, input)
}

fn into_expr(pairs: Pairs<Rule>) -> ResA<AstExpr> {
    use AstExpr::*;
    PC_EXPR.climb(
        pairs,
        |pair: Pair<Rule>| match pair.as_rule() {
            Rule::binop_expr => into_expr(pair.into_inner()),
            Rule::lit_bool => pair.as_str().trim().parse().map(CstB).context("Parsing a bool literal"),
            Rule::lit_str => {
                let str = pair.as_str().trim();
                let len = str.len();
                let inner = str[1..len - 2].into();
                Ok(CstS(inner))
            }
            Rule::lit_num => pair.as_str().trim().parse().map(CstN).context("Parsing a numeric literal"),
            _ => unreachable!(),
        },
        |lhs: ResA<AstExpr>, op: Pair<Rule>, rhs: ResA<AstExpr>| {
            let binop = match op.as_rule() {
                Rule::binop_add => BinopT::Add,
                Rule::binop_sub => BinopT::Sub,
                Rule::binop_mul => BinopT::Mul,
                Rule::binop_div => BinopT::Div,
                Rule::binop_and => BinopT::And,
                Rule::binop_or => BinopT::Or,
                _ => unreachable!()
            };
            Ok(Binop(binop, Box::from(lhs?), Box::from(rhs?)))
        },
    )
}
