use crate::parse::Token;
use crate::utils::LRange;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct NodeData<T> {
    pub data: T,
    pub subdir: String,
    pub range: LRange,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Node<'a> {
    IdentNode(NodeData<&'a str>),
    NumberNode(NodeData<f64>),
    StringNode(NodeData<&'a str>),
    BooleanNode(NodeData<bool>),
    ContinueNode(NodeData<()>),
    BreakNode(NodeData<()>),
    ArrayNode(NodeData<Box<Node<'a>>>),
    EmptyNode(NodeData<()>),
    OrNode(NodeData<(Box<Node<'a>>, Box<Node<'a>>)>),
    AndNode(NodeData<(Box<Node<'a>>, Box<Node<'a>>)>),
    NotNode(NodeData<(Box<Node<'a>>)>),
    ComparisonNode(NodeData<(Comparison, Box<Node<'a>>, Box<Node<'a>>)>),
    ArithmeticNode(NodeData<(Arithmetic, Box<Node<'a>>, Box<Node<'a>>)>),
    IndexNode {
        object: Box<Node<'a>>,
        index: Box<Node<'a>>
    },
    MethodName {
        name: &'a str,
        source_object: Node<'a>
    },
    FonctionNode {
        method: Box<Node<'a>>,
        args: Box<Node<'a>>
    }
}