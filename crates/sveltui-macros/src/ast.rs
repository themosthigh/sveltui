use syn::{Expr, Ident, LitStr, Pat};

pub enum Node {
    Element(Element),
    Text(LitStr),
    Block(Expr),
    If(IfNode),
    Each(EachNode),
    Await(AwaitNode),
    Snippet(SnippetNode),
    Render(RenderNode),
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

pub enum IfCondition {
    Expr(Expr),
    Let { pat: Pat, expr: Expr },
}

pub struct IfBranch {
    pub condition: IfCondition,
    pub body: Vec<Node>,
}

pub struct IfNode {
    pub initial: IfBranch,
    pub else_ifs: Vec<IfBranch>,
    pub else_branch: Option<Vec<Node>>,
}

pub struct EachNode {
    pub expr: Expr,
    pub pat: Pat,
    pub key: Option<Expr>,
    pub body: Vec<Node>,
    pub else_branch: Option<Vec<Node>>,
}

pub struct AwaitNode {
    pub future: Expr,
    pub loading: Vec<Node>,
    pub then_branch: Option<(Pat, Vec<Node>)>,
    pub catch_branch: Option<(Pat, Vec<Node>)>,
}

pub struct SnippetNode {
    pub name: Ident,
    pub args: Vec<Pat>,
    pub body: Vec<Node>,
}

pub struct RenderNode {
    pub target: Expr,
}

pub struct ViewTree {
    pub root: Node,
}
