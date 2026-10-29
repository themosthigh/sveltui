use crossterm::style::Color;
use std::rc::Rc;
use svtui::prelude::*;

#[derive(Props)]
pub struct DialogProps {
    #[prop(default = false)]
    pub open: bool,
    #[prop(optional)]
    pub id: Option<FocusId>,
    #[prop(optional)]
    pub opener: Option<FocusId>,
    #[prop(default = InitialFocus::First)]
    pub initial_focus: InitialFocus,
    #[prop(optional)]
    pub title: Option<String>,
    #[prop(default = BorderStyle::Single)]
    pub border: BorderStyle,
    #[prop(default = Color::White)]
    pub border_color: Color,
    #[prop(default = Size::Fixed(50))]
    pub width: Size,
    #[prop(default = Size::Auto)]
    pub height: Size,
    #[prop(default = Padding::all(1))]
    pub padding: Padding,
    #[prop(optional)]
    pub on_close: Option<Rc<dyn Fn()>>,
    #[prop(optional)]
    pub on_open: Option<Rc<dyn Fn()>>,
    #[prop(optional)]
    pub on_key: Option<KeyHandler>,
    #[prop(default = Placement::Center)]
    pub placement: Placement,
    #[prop(default = true)]
    pub backdrop: bool,
    #[snippet(optional)]
    pub children: Option<Snippet>,
}

#[component]
pub fn Dialog(props: DialogProps) -> impl IntoNode {
    let id = props.id.unwrap_or_default();
    let mut layer = DismissLayer::dialog();
    layer.scope.initial_focus = props.initial_focus;

    let title_str = props.title.unwrap_or_default();
    let semantics = Semantics {
        modal: Some(true),
        ..Semantics::new(Role::Dialog, title_str.clone())
    };

    view! {
        <Overlay
            id=id
            open=props.open
            placement=props.placement
            layer=layer
            backdrop=props.backdrop
            modal=true
            opt_opener=props.opener
            opt_on_close=props.on_close
            opt_on_open=props.on_open
            opt_on_key=props.on_key
            semantics=semantics
        >
            <FlexBox
                border=props.border
                border_color=props.border_color
                width=props.width
                height=props.height
                padding=props.padding
                opt_title={if title_str.is_empty() { None } else { Some(title_str) }}
            >
                {#if props.children.is_some()}
                    {@render props.children}
                {/if}
            </FlexBox>
        </Overlay>
    }
}
