use crate::utils::LRange;

#[derive(Clone, Debug, PartialEq)]
pub struct NodeData<T> {
    pub data: T,
    pub subdir: String,
    pub range: LRange,
}

type MemberAccess = Vec<String>;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum BinopT {
    Add,
    Sub,
    Mul,
    Div,
    And,
    Or,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum UnopT {
    Neg,
    Not,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AstExpr {
    CstB(bool),
    CstS(String),
    CstN(f64),
    Binop(BinopT, Box<AstExpr>, Box<AstExpr>),
    Unop(UnopT, Box<AstExpr>),
    Call(MemberAccess, Vec<AstExpr>),
}
