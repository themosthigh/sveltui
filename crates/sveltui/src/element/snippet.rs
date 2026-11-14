use crate::element::node::{IntoNode, VNode};
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

/// The builder state of a required prop that has been given a value.
/// A derived `Props` builder only has `build()` once every required prop
/// is in this state.
pub struct PropSet;

/// Marks a required prop as set, so `build()` can be called. A builder
/// that is missing one doesn't compile:
///
/// ```compile_fail
/// use sveltui::prelude::*;
///
/// #[derive(Props)]
/// struct Card {
///     title: String,
/// }
///
/// let _ = Card::builder().build();
/// ```
#[diagnostic::on_unimplemented(
    message = "a required prop is missing",
    label = "this prop must be set before the component can be built",
    note = "the type name `__<Props>_missing_<prop>` in this error says which prop"
)]
pub trait PropIsSet {}

impl PropIsSet for PropSet {}

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

impl IntoPropOption<crate::element::focus::FocusId> for crate::element::focus::FocusId {
    fn into_prop_option(self) -> Option<crate::element::focus::FocusId> {
        Some(self)
    }
}

impl IntoPropOption<crate::layout::BorderStyle> for crate::layout::BorderStyle {
    fn into_prop_option(self) -> Option<crate::layout::BorderStyle> {
        Some(self)
    }
}

impl IntoPropOption<crossterm::style::Color> for crossterm::style::Color {
    fn into_prop_option(self) -> Option<crossterm::style::Color> {
        Some(self)
    }
}

impl IntoPropOption<crate::layout::Dimension> for crate::layout::Dimension {
    fn into_prop_option(self) -> Option<crate::layout::Dimension> {
        Some(self)
    }
}

impl IntoPropOption<crate::layout::Dimension> for u16 {
    fn into_prop_option(self) -> Option<crate::layout::Dimension> {
        Some(crate::layout::Dimension::Length(f32::from(self)))
    }
}

impl IntoPropOption<crate::layout::Dimension> for i32 {
    fn into_prop_option(self) -> Option<crate::layout::Dimension> {
        Some(crate::layout::Dimension::Length(self.max(0) as f32))
    }
}

impl IntoPropOption<crate::layout::Dimension> for f32 {
    fn into_prop_option(self) -> Option<crate::layout::Dimension> {
        Some(crate::layout::IntoDimension::into_dimension(self))
    }
}

impl IntoPropOption<crate::layout::Dimension> for f64 {
    fn into_prop_option(self) -> Option<crate::layout::Dimension> {
        Some(crate::layout::IntoDimension::into_dimension(self))
    }
}

impl IntoPropOption<crate::layout::Padding> for u16 {
    fn into_prop_option(self) -> Option<crate::layout::Padding> {
        Some(crate::layout::Padding::all(self))
    }
}

impl IntoPropOption<crate::layout::Padding> for i32 {
    fn into_prop_option(self) -> Option<crate::layout::Padding> {
        Some(crate::layout::Padding::all(self as u16))
    }
}

impl IntoPropOption<crate::layout::Padding> for crate::layout::Padding {
    fn into_prop_option(self) -> Option<crate::layout::Padding> {
        Some(self)
    }
}

impl IntoPropOption<crate::layout::Margin> for u16 {
    fn into_prop_option(self) -> Option<crate::layout::Margin> {
        Some(crate::layout::Margin::all(self))
    }
}

impl IntoPropOption<crate::layout::Margin> for i32 {
    fn into_prop_option(self) -> Option<crate::layout::Margin> {
        Some(crate::layout::Margin::all(self as u16))
    }
}

impl IntoPropOption<crate::layout::Margin> for crate::layout::Margin {
    fn into_prop_option(self) -> Option<crate::layout::Margin> {
        Some(self)
    }
}

impl IntoPropOption<crate::layout::AlignItems> for crate::layout::AlignItems {
    fn into_prop_option(self) -> Option<crate::layout::AlignItems> {
        Some(self)
    }
}

impl IntoPropOption<crate::layout::JustifyContent> for crate::layout::JustifyContent {
    fn into_prop_option(self) -> Option<crate::layout::JustifyContent> {
        Some(self)
    }
}

impl IntoPropOption<taffy::style::FlexDirection> for taffy::style::FlexDirection {
    fn into_prop_option(self) -> Option<taffy::style::FlexDirection> {
        Some(self)
    }
}

impl IntoPropOption<taffy::style::FlexWrap> for taffy::style::FlexWrap {
    fn into_prop_option(self) -> Option<taffy::style::FlexWrap> {
        Some(self)
    }
}

impl IntoPropOption<taffy::style::Overflow> for taffy::style::Overflow {
    fn into_prop_option(self) -> Option<taffy::style::Overflow> {
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

/// One-argument callbacks, with or without a result (`on_change`,
/// `is_date_unavailable`).
impl<A: 'static, R: 'static, F: Fn(A) -> R + 'static> IntoPropOption<Rc<dyn Fn(A) -> R>> for F {
    fn into_prop_option(self) -> Option<Rc<dyn Fn(A) -> R>> {
        Some(Rc::new(self))
    }
}
impl<A: 'static, R: 'static> IntoPropOption<Rc<dyn Fn(A) -> R>> for Rc<dyn Fn(A) -> R> {
    fn into_prop_option(self) -> Option<Rc<dyn Fn(A) -> R>> {
        Some(self)
    }
}
impl IntoPropOption<Option<String>> for Option<String> {
    fn into_prop_option(self) -> Option<Option<String>> {
        Some(self)
    }
}
impl IntoPropOption<crate::layout::Placement> for crate::layout::Placement {
    fn into_prop_option(self) -> Option<crate::layout::Placement> {
        Some(self)
    }
}

/// Implements `IntoPropOption<T> for T` for each type listed. Together with the
/// blanket `Option<T>` impl, that lets one setter take either `T` or
/// `Option<T>`.
macro_rules! identity_prop_option {
    ($($ty:ty),* $(,)?) => {$(
        impl IntoPropOption<$ty> for $ty {
            fn into_prop_option(self) -> Option<$ty> {
                Some(self)
            }
        }
    )*};
}

identity_prop_option!(
    (u16, u16),
    crate::terminal::CursorStyle,
    crate::element::interaction::Semantics,
    crate::element::interaction::FocusScope,
    crate::element::interaction::FocusGroup,
    crate::element::interaction::Interaction,
    crate::element::text_buffer::TextBuffer,
);

impl IntoPropOption<crate::element::focus::FocusId> for usize {
    fn into_prop_option(self) -> Option<crate::element::focus::FocusId> {
        Some(self.into())
    }
}

impl<T> IntoPropOption<crate::reactive::ReadSignal<T>> for crate::reactive::ReadSignal<T> {
    fn into_prop_option(self) -> Option<crate::reactive::ReadSignal<T>> {
        Some(self)
    }
}

impl<T> IntoPropOption<crate::reactive::WriteSignal<T>> for crate::reactive::WriteSignal<T> {
    fn into_prop_option(self) -> Option<crate::reactive::WriteSignal<T>> {
        Some(self)
    }
}

impl<T> IntoPropOption<crate::reactive::State<T>> for crate::reactive::State<T> {
    fn into_prop_option(self) -> Option<crate::reactive::State<T>> {
        Some(self)
    }
}

impl<T> IntoPropOption<std::collections::HashSet<T>> for std::collections::HashSet<T> {
    fn into_prop_option(self) -> Option<std::collections::HashSet<T>> {
        Some(self)
    }
}
