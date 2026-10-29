use crossterm::event::{KeyCode, KeyEvent};
use crossterm::style::Color;
use std::rc::Rc;
use svtui::prelude::*;

#[derive(Clone)]
pub enum TextInputBinding {
    Read(ReadSignal<String>),
    State(State<String>),
}

impl From<ReadSignal<String>> for TextInputBinding {
    fn from(s: ReadSignal<String>) -> Self {
        Self::Read(s)
    }
}

impl From<State<String>> for TextInputBinding {
    fn from(s: State<String>) -> Self {
        Self::State(s)
    }
}

impl IntoPropOption<TextInputBinding> for TextInputBinding {
    fn into_prop_option(self) -> Option<TextInputBinding> {
        Some(self)
    }
}

impl IntoPropOption<TextInputBinding> for ReadSignal<String> {
    fn into_prop_option(self) -> Option<TextInputBinding> {
        Some(TextInputBinding::Read(self))
    }
}

impl IntoPropOption<TextInputBinding> for State<String> {
    fn into_prop_option(self) -> Option<TextInputBinding> {
        Some(TextInputBinding::State(self))
    }
}

pub fn get_cursor(id: FocusId, text_len: usize) -> usize {
    with_buffer_mut(id, |buf| {
        if buf.cursor_col > text_len {
            buf.cursor_col = text_len;
        }
        buf.cursor_col
    })
}

pub fn set_cursor(id: FocusId, pos: usize) {
    with_buffer_mut(id, |buf| {
        buf.cursor_col = pos;
    });
}

#[derive(Props)]
pub struct TextInputProps {
    #[prop(optional)]
    pub id: Option<FocusId>,
    #[prop(optional)]
    pub value: Option<TextInputBinding>,
    #[prop(optional)]
    pub set_value: Option<WriteSignal<String>>,
    #[prop(optional)]
    pub buffer: Option<ReadSignal<TextBuffer>>,
    #[prop(optional)]
    pub set_buffer: Option<WriteSignal<TextBuffer>>,
    #[prop(default = String::from("Type here..."))]
    pub placeholder: String,
    #[prop(default = Size::Fixed(24))]
    pub width: Size,
    #[prop(optional)]
    pub title: Option<String>,
    #[prop(optional)]
    pub on_submit: Option<Rc<dyn Fn(String)>>,
}

#[component]
pub fn TextInput(props: TextInputProps) -> impl IntoNode {
    let id = props.id.unwrap_or_default();
    let focused = is_focused(id);

    let border_style = if focused {
        BorderStyle::Double
    } else {
        BorderStyle::Single
    };

    let border_color = if focused {
        Color::White
    } else {
        Color::DarkGrey
    };

    let visible_width = match props.width {
        Size::Fixed(w) => (w.saturating_sub(4)).max(1) as usize,
        _ => 20,
    };

    let (val_sig, binding_set_val) = match props.value {
        Some(TextInputBinding::Read(r)) => (Some(r), None),
        Some(TextInputBinding::State(s)) => (Some(s.read()), Some(s.write())),
        None => (None, None),
    };

    let set_value = props.set_value.or(binding_set_val);

    let (display_text, text_color, cursor_col) = with_buffer_mut(id, |buf| {
        buf.set_multiline(false);

        if let Some(buf_sig) = props.buffer {
            *buf = buf_sig.get();
            buf.set_multiline(false);
        } else if let Some(vs) = val_sig {
            let current_val = vs.get();
            if buf.content() != current_val {
                buf.set_content(current_val);
            }
        }

        buf.update_viewport(visible_width, 1);
        let cur_col = (buf.cursor_col.saturating_sub(buf.scroll_col)) as u16;

        if buf.is_empty() {
            let ph = if props.placeholder.is_empty() {
                String::from("Type here...")
            } else {
                props.placeholder.clone()
            };
            (ph, Color::DarkGrey, 0u16)
        } else {
            let visible = buf.visible_text(visible_width);
            (visible, Color::White, cur_col)
        }
    });

    let set_buffer = props.set_buffer;
    let on_submit = props.on_submit;

    let on_click_at: Rc<dyn Fn(u16, u16)> = Rc::new(move |rel_x: u16, _rel_y: u16| {
        let click_col = rel_x.saturating_sub(2) as usize;
        with_buffer_mut(id, |buf| {
            let new_cursor = buf.scroll_col + click_col;
            buf.set_cursor(new_cursor);
            if let Some(sb) = set_buffer {
                sb.set(buf.clone());
            }
        });
    });

    let on_key: KeyHandler = Rc::new(move |key: KeyEvent, _ctx: &mut EventContext| {
        if key.code == KeyCode::Enter {
            if let Some(ref submit) = on_submit {
                let text = with_buffer(id, |b| b.content());
                submit(text);
            }
            return if on_submit.is_some() {
                EventResult::Handled
            } else {
                EventResult::Ignored
            };
        }

        if !with_buffer(id, |buf| buf.accepts_key(&key)) {
            return EventResult::Ignored;
        }
        with_buffer_mut(id, |buf| {
            let modified = buf.handle_key(&key);
            if modified {
                let text = buf.content();
                if let Some(sv) = set_value {
                    sv.set(text);
                }
                if let Some(sb) = set_buffer {
                    sb.set(buf.clone());
                }
            }
        });
        EventResult::Handled
    });

    let cursor_offset = if focused { Some((cursor_col, 0)) } else { None };
    let title_str = props
        .title
        .clone()
        .unwrap_or_else(|| props.placeholder.clone());

    view! {
        <FlexBox
            border=border_style
            border_color=border_color
            padding=Padding::symmetric(1, 0)
            width=props.width
            justify_content=Justify::Center
            align_items=Align::Start
            focus_id=id
            tab_stop=true
            semantics=Semantics::new(Role::Textbox, title_str)
            on_click_at=on_click_at
            on_key=on_key
            opt_title=props.title
            opt_cursor_offset=cursor_offset
        >
            <TextSpan fg=text_color bold=focused>{display_text}</TextSpan>
        </FlexBox>
    }
}
