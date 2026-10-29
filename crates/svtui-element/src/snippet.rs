use crate::node::{IntoNode, VNode};
use std::fmt;
use std::rc::Rc;

pub trait Props: Sized {
    type Builder: Default;
}

#[derive(Clone)]
pub struct Snippet<T = ()>(Option<Rc<dyn Fn(T) -> VNode>>);

impl<T> fmt::Debug for Snippet<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Snippet")
            .field("is_some", &self.is_some())
            .finish()
    }
}

impl<T> Default for Snippet<T> {
    fn default() -> Self {
        Self::none()
    }
}

impl<T> Snippet<T> {
    pub fn new<N: IntoNode>(f: impl Fn(T) -> N + 'static) -> Self {
        Self(Some(Rc::new(move |arg| f(arg).into_node())))
    }

    pub fn none() -> Self {
        Self(None)
    }

    pub fn is_some(&self) -> bool {
        self.0.is_some()
    }

    pub fn is_none(&self) -> bool {
        self.0.is_none()
    }

    pub fn is_snippet_present(&self) -> bool {
        self.is_some()
    }

    pub fn render(&self, arg: T) -> VNode {
        match &self.0 {
            Some(f) => f(arg),
            None => VNode::Empty,
        }
    }
}

impl Snippet<()> {
    pub fn render_empty(&self) -> VNode {
        self.render(())
    }
}

impl IntoNode for Snippet<()> {
    fn into_node(self) -> VNode {
        self.render(())
    }
}

impl IntoNode for &Snippet<()> {
    fn into_node(self) -> VNode {
        self.render(())
    }
}

impl From<&'static str> for Snippet<()> {
    fn from(s: &'static str) -> Self {
        Self::new(move |_| s)
    }
}

impl From<String> for Snippet<()> {
    fn from(s: String) -> Self {
        Self::new(move |_| s.clone())
    }
}

impl From<VNode> for Snippet<()> {
    fn from(node: VNode) -> Self {
        Self::new(move |_| node.clone())
    }
}

pub trait OptionSnippetExt<T> {
    fn is_snippet_present(&self) -> bool;
    fn render(&self, arg: T) -> VNode;
}

impl<T> OptionSnippetExt<T> for Option<Snippet<T>> {
    fn is_snippet_present(&self) -> bool {
        match self {
            Some(s) => s.is_some(),
            None => false,
        }
    }

    fn render(&self, arg: T) -> VNode {
        match self {
            Some(s) => s.render(arg),
            None => VNode::Empty,
        }
    }
}

impl<T> OptionSnippetExt<T> for Option<&Snippet<T>> {
    fn is_snippet_present(&self) -> bool {
        match self {
            Some(s) => s.is_some(),
            None => false,
        }
    }

    fn render(&self, arg: T) -> VNode {
        match self {
            Some(s) => s.render(arg),
            None => VNode::Empty,
        }
    }
}

pub trait IntoPropOption<T> {
    fn into_prop_option(self) -> Option<T>;
}

impl<T> IntoPropOption<T> for Option<T> {
    fn into_prop_option(self) -> Option<T> {
        self
    }
}

impl IntoPropOption<String> for String {
    fn into_prop_option(self) -> Option<String> {
        Some(self)
    }
}

impl IntoPropOption<String> for &str {
    fn into_prop_option(self) -> Option<String> {
        Some(self.to_string())
    }
}

impl IntoPropOption<String> for &String {
    fn into_prop_option(self) -> Option<String> {
        Some(self.clone())
    }
}

impl IntoPropOption<String> for Option<&str> {
    fn into_prop_option(self) -> Option<String> {
        self.map(|s| s.to_string())
    }
}

impl<T> IntoPropOption<Snippet<T>> for Snippet<T> {
    fn into_prop_option(self) -> Option<Snippet<T>> {
        if self.is_some() { Some(self) } else { None }
    }
}

impl IntoPropOption<Snippet<()>> for &'static str {
    fn into_prop_option(self) -> Option<Snippet<()>> {
        Some(Snippet::from(self))
    }
}

impl IntoPropOption<Snippet<()>> for String {
    fn into_prop_option(self) -> Option<Snippet<()>> {
        Some(Snippet::from(self))
    }
}

impl IntoPropOption<Snippet<()>> for VNode {
    fn into_prop_option(self) -> Option<Snippet<()>> {
        Some(Snippet::from(self))
    }
}

impl IntoPropOption<Snippet<()>> for Option<VNode> {
    fn into_prop_option(self) -> Option<Snippet<()>> {
        self.map(Snippet::from)
    }
}

impl<F: Fn() + 'static> IntoPropOption<Rc<dyn Fn()>> for F {
    fn into_prop_option(self) -> Option<Rc<dyn Fn()>> {
        Some(Rc::new(self))
    }
}

impl IntoPropOption<Rc<dyn Fn()>> for Rc<dyn Fn()> {
    fn into_prop_option(self) -> Option<Rc<dyn Fn()>> {
        Some(self)
    }
}

impl IntoPropOption<crate::focus::FocusId> for crate::focus::FocusId {
    fn into_prop_option(self) -> Option<crate::focus::FocusId> {
        Some(self)
    }
}

impl IntoPropOption<svtui_layout::BorderStyle> for svtui_layout::BorderStyle {
    fn into_prop_option(self) -> Option<svtui_layout::BorderStyle> {
        Some(self)
    }
}

impl IntoPropOption<crossterm::style::Color> for crossterm::style::Color {
    fn into_prop_option(self) -> Option<crossterm::style::Color> {
        Some(self)
    }
}

impl IntoPropOption<svtui_layout::Size> for svtui_layout::Size {
    fn into_prop_option(self) -> Option<svtui_layout::Size> {
        Some(self)
    }
}

impl IntoPropOption<svtui_layout::Size> for u16 {
    fn into_prop_option(self) -> Option<svtui_layout::Size> {
        Some(svtui_layout::Size::Fixed(self))
    }
}

impl IntoPropOption<svtui_layout::Size> for i32 {
    fn into_prop_option(self) -> Option<svtui_layout::Size> {
        Some(svtui_layout::Size::Fixed(self as u16))
    }
}

impl IntoPropOption<svtui_layout::Padding> for u16 {
    fn into_prop_option(self) -> Option<svtui_layout::Padding> {
        Some(svtui_layout::Padding::all(self))
    }
}

impl IntoPropOption<svtui_layout::Padding> for i32 {
    fn into_prop_option(self) -> Option<svtui_layout::Padding> {
        Some(svtui_layout::Padding::all(self as u16))
    }
}

impl IntoPropOption<svtui_layout::Padding> for svtui_layout::Padding {
    fn into_prop_option(self) -> Option<svtui_layout::Padding> {
        Some(self)
    }
}

impl IntoPropOption<bool> for bool {
    fn into_prop_option(self) -> Option<bool> {
        Some(self)
    }
}

impl IntoPropOption<u32> for u32 {
    fn into_prop_option(self) -> Option<u32> {
        Some(self)
    }
}

impl IntoPropOption<i32> for i32 {
    fn into_prop_option(self) -> Option<i32> {
        Some(self)
    }
}

impl IntoPropOption<u16> for u16 {
    fn into_prop_option(self) -> Option<u16> {
        Some(self)
    }
}

impl IntoPropOption<f32> for f32 {
    fn into_prop_option(self) -> Option<f32> {
        Some(self)
    }
}

impl IntoPropOption<f64> for f64 {
    fn into_prop_option(self) -> Option<f64> {
        Some(self)
    }
}

impl IntoPropOption<usize> for usize {
    fn into_prop_option(self) -> Option<usize> {
        Some(self)
    }
}
