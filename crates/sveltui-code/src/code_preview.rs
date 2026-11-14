use crate::highlight::highlight_text;
use crossterm::event::{KeyCode, KeyEvent};
use crossterm::style::Color;
use std::rc::Rc;
use sveltui::element::scroll::{get_scroll, set_scroll};
use sveltui::layout::Dimension;
use sveltui::prelude::*;

#[derive(Props)]
pub struct CodePreviewProps {
    #[prop(optional)]
    pub id: Option<FocusId>,
    /// The code to show. `code` does the same and wins when both are set.
    #[prop(optional)]
    pub value: Option<String>,
    #[prop(optional)]
    pub code: Option<String>,
    #[prop(optional)]
    pub language: Option<String>,
    #[prop(optional)]
    pub theme: Option<String>,
    #[prop(default = true)]
    pub show_line_numbers: bool,
    #[prop(default = Dimension::Auto)]
    pub width: Dimension,
    #[prop(default = Dimension::Auto)]
    pub height: Dimension,
    #[prop(default = BorderStyle::Single)]
    pub border: BorderStyle,
    #[prop(optional)]
    pub border_color: Option<Color>,
    #[prop(optional)]
    pub title: Option<String>,
    #[prop(default = 4)]
    pub tab_size: usize,
}

#[component]
pub fn CodePreview(props: CodePreviewProps) -> impl IntoNode {
    let id = props.id.unwrap_or_default();
    let focused = is_focused(id);

    let border_style = if focused {
        BorderStyle::Double
    } else {
        props.border
    };

    let border_color = if focused {
        Color::White
    } else {
        props.border_color.unwrap_or(Color::DarkGrey)
    };

    let raw_text = if let Some(ref c) = props.code {
        c.clone()
    } else if let Some(ref value) = props.value {
        value.clone()
    } else {
        String::new()
    };

    let all_line_spans =
        highlight_text(&raw_text, props.language.as_deref(), props.theme.as_deref());

    let total_lines = all_line_spans.len().max(1);
    let digits = format!("{}", total_lines).len().max(2);
    let show_line_numbers = props.show_line_numbers;

    let rendered_line_nodes: Vec<VNode> = all_line_spans
        .into_iter()
        .enumerate()
        .map(|(offset, spans)| {
            let line_num = offset + 1;
            let mut row = ViewProps::builder()
                .flex_direction(FlexDirection::Row)
                .align_items(AlignItems::Center)
                .justify_content(JustifyContent::FlexStart);
            if show_line_numbers {
                row = row.child(TextSpan(
                    TextSpanProps::builder()
                        .foreground_color(Color::DarkGrey)
                        .content(format!("{:>width$} │ ", line_num, width = digits))
                        .build(),
                ));
            }
            if spans.is_empty() {
                row = row.child(TextSpan(
                    TextSpanProps::builder()
                        .foreground_color(Color::DarkGrey)
                        .content(" ")
                        .build(),
                ));
            } else {
                for span in spans {
                    row = row.child(TextSpan(
                        TextSpanProps::builder()
                            .foreground_color(span.fg)
                            .bold(span.bold)
                            .italic(span.italic)
                            .underline(span.underline)
                            .content(span.text)
                            .build(),
                    ));
                }
            }
            View(row.build()).into_node()
        })
        .collect();

    let title_str = props.title.clone().unwrap_or_else(|| {
        if let Some(ref lang) = props.language {
            format!("Preview ({lang})")
        } else {
            String::from("Code Preview")
        }
    });

    let on_key: KeyHandler = Rc::new(move |key: KeyEvent, _ctx: &mut EventContext| {
        let (sx, sy) = get_scroll(id);
        let page_step = 10;

        match key.code {
            KeyCode::Up => {
                set_scroll(id, (sx, sy.saturating_sub(1)));
                EventResult::Handled
            }
            KeyCode::Down => {
                set_scroll(id, (sx, sy.saturating_add(1)));
                EventResult::Handled
            }
            KeyCode::PageUp => {
                set_scroll(id, (sx, sy.saturating_sub(page_step)));
                EventResult::Handled
            }
            KeyCode::PageDown => {
                set_scroll(id, (sx, sy.saturating_add(page_step)));
                EventResult::Handled
            }
            KeyCode::Home => {
                set_scroll(id, (0, 0));
                EventResult::Handled
            }
            KeyCode::End => {
                set_scroll(id, (sx, u16::MAX));
                EventResult::Handled
            }
            KeyCode::Left => {
                set_scroll(id, (sx.saturating_sub(2), sy));
                EventResult::Handled
            }
            KeyCode::Right => {
                set_scroll(id, (sx.saturating_add(2), sy));
                EventResult::Handled
            }
            _ => EventResult::Ignored,
        }
    });

    let on_scroll: Rc<dyn Fn(i16, i16)> = Rc::new(move |_dx: i16, dy: i16| {
        let (sx, sy) = get_scroll(id);
        let ny = if dy > 0 {
            sy.saturating_add(dy as u16)
        } else {
            sy.saturating_sub((-dy) as u16)
        };
        set_scroll(id, (sx, ny));
    });

    view! {
        <Pressable
            focus_id=id
            tab_stop=true
            on_key=on_key
            semantics={Semantics::new(Role::Generic, title_str)}
        >
            <View
                border=border_style
                border_color=border_color
                padding={Padding::symmetric(1, 0)}
                width={props.width}
                height={props.height}
                flex_direction=FlexDirection::Column
                align_items=AlignItems::FlexStart
                justify_content=JustifyContent::FlexStart
                overflow_x=Overflow::Scroll
                overflow_y=Overflow::Scroll
                scroll_id=id
                on_scroll=on_scroll
                title={props.title}
            >
                {#each rendered_line_nodes as line_node}
                    {line_node}
                {/each}
            </View>
        </Pressable>
    }
}
