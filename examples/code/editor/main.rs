use crossterm::event::{KeyCode, KeyEvent};
use std::cell::RefCell;
use sveltui::app::exit_shortcuts;
use sveltui::prelude::*;
use sveltui_code::prelude::*;

thread_local! {
    static CURSOR_TOGGLE: RefCell<Option<WriteSignal<CursorStyle>>> = const { RefCell::new(None) };
}

#[component]
fn CodeEditorDemo() -> impl IntoNode {
    let (cursor_style, set_cursor_style) = create_signal(CursorStyle::Line);

    // Register cursor toggle signal for global shortcut handling
    CURSOR_TOGGLE.with(|c| *c.borrow_mut() = Some(set_cursor_style));

    let toggle_cursor = move || {
        set_cursor_style.update(|s| {
            *s = if *s == CursorStyle::Line {
                CursorStyle::Block
            } else {
                CursorStyle::Line
            };
        });
    };

    let code = state(String::from(
        r#"// Interactive Code Editor with TextMate Highlighting!
fn fibonacci(n: u32) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

fn main() {
    let result = fibonacci(10);
    println!("Fibonacci(10) = {}", result);
}
"#,
    ));

    let cursor_badge_text = match cursor_style.get() {
        CursorStyle::Block => " Cursor: Block (F2 to toggle) ",
        _ => " Cursor: Line (F2 to toggle) ",
    };

    view! {
        <View
            width={Dimension::Percent(1.0)}
            height={Dimension::Percent(1.0)}
            flex_direction=FlexDirection::Column
            padding={Padding::all(1)}
            gap=1
            overflow=Overflow::Hidden
        >
            <View
                flex_direction=FlexDirection::Row
                justify_content=JustifyContent::SpaceBetween
                align_items=AlignItems::Center
                width={Dimension::Percent(1.0)}
            >
                <TextSpan foreground_color=Color::DarkGrey>
                    "sveltui Code Editor  •  Arrow keys navigate  •  Esc to quit"
                </TextSpan>
                <Pressable on_click=toggle_cursor>
                    <View
                        border=BorderStyle::Single
                        border_color=Color::Cyan
                        align_items=AlignItems::Center
                        justify_content=JustifyContent::Center
                    >
                        <TextSpan foreground_color=Color::Cyan bold=true>
                            {cursor_badge_text}
                        </TextSpan>
                    </View>
                </Pressable>
            </View>

            <View
                flex_direction=FlexDirection::Row
                width={Dimension::Percent(1.0)}
                height={Dimension::Percent(1.0)}
                gap=2
                overflow=Overflow::Hidden
            >
                <View
                    flex_direction=FlexDirection::Column
                    width={Dimension::Percent(0.5)}
                    height={Dimension::Percent(1.0)}
                    gap=0
                >
                    <CodeEditor
                        bind:value=code
                        language="rs"
                        show_line_numbers=true
                        width={Dimension::Percent(1.0)}
                        height={Dimension::Percent(1.0)}
                        cursor_style={cursor_style.get()}
                        title="editor.rs (Editable)"
                    />
                </View>

                <View
                    flex_direction=FlexDirection::Column
                    width={Dimension::Percent(0.5)}
                    height={Dimension::Percent(1.0)}
                    gap=0
                >
                    <CodePreview
                        value={code.get()}
                        language="rs"
                        show_line_numbers=true
                        width={Dimension::Percent(1.0)}
                        height={Dimension::Percent(1.0)}
                        title="preview.rs (Live Preview)"
                    />
                </View>
            </View>
        </View>
    }
}

fn main() -> std::io::Result<()> {
    App::new(|| view! { <CodeEditorDemo /> })
        .on_key(|key: KeyEvent, ctx: &mut EventContext| {
            if key.code == KeyCode::F(2) {
                CURSOR_TOGGLE.with(|c| {
                    if let Some(set_cursor) = *c.borrow() {
                        set_cursor.update(|s| {
                            *s = if *s == CursorStyle::Line {
                                CursorStyle::Block
                            } else {
                                CursorStyle::Line
                            };
                        });
                    }
                });
                return EventResult::Handled;
            }
            exit_shortcuts(key, ctx)
        })
        .run()
}
