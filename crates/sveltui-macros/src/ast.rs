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
    /// Written as `bind:name`, so it calls the component's `bind_name` prop.
    pub bind: bool,
}

pub struct Element {
    pub name: syn::Path,
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
    /// The `i` in `{#each xs as item, i}`.
    pub index: Option<Ident>,
    pub key: Option<Expr>,
    pub body: Vec<Node>,
    pub else_branch: Option<Vec<Node>>,
}

pub struct AwaitNode {
    pub future: Expr,
    pub loading: Vec<Node>,
    /// A `None` pattern is a bare `{:then}`: show the body, bind nothing.
    pub then_branch: Option<(Option<Pat>, Vec<Node>)>,
    pub catch_branch: Option<(Option<Pat>, Vec<Node>)>,
}

pub struct SnippetNode {
    pub name: Ident,
    pub args: Vec<Pat>,
    pub body: Vec<Node>,
}

pub struct RenderNode {
    pub target: Expr,
    /// Written as `{@render x?.(..)}`: render nothing when `x` isn't set.
    pub optional: bool,
}

pub struct ViewTree {
    pub root: Node,
}
