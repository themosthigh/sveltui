use syn::{Expr, Ident, LitStr};

pub enum Node {
    Element(Element),
    Text(LitStr),
    Block(Expr),
}

pub struct Attribute {
    pub name: Ident,
    pub value: Expr,
}

pub struct Element {
    pub name: Ident,
    pub attributes: Vec<Attribute>,
    pub children: Vec<Node>,
}

pub struct ViewTree {
    pub root: Node,
}
