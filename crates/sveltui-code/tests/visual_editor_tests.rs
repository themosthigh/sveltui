use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use sveltui::Buffer;
use sveltui::element::scroll::get_scroll;
use sveltui::element::{render_node_interactive, reset_focus_counter};
use sveltui::layout::Rect;
use sveltui::prelude::*;
use sveltui_code::prelude::*;

pub struct TestTerminal {
    manager: FocusManager,
    buffer: Buffer,
    hits: HitMap,
    width: u16,
    height: u16,
}

impl TestTerminal {
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            manager: FocusManager::new(),
            buffer: Buffer::new(width, height),
            hits: HitMap::new(),
            width,
            height,
        }
    }

    pub fn draw(&mut self, build: &impl Fn() -> VNode) {
        for _ in 0..5 {
            reset_focus_counter();
            begin_frame();
            let tree = self.manager.with_context(build);
            end_frame();

            self.buffer.clear();
            self.hits.clear();

            let screen_rect = Rect::new(0, 0, self.width, self.height);
            self.manager.with_context(|| {
                render_node_interactive(&tree, screen_rect, &mut self.buffer, &mut self.hits)
            });

            let changed = self.manager.reconcile(self.hits.interactions.clone());
            let revealed = self
                .manager
                .focused()
                .is_some_and(|id| self.hits.reveal(id));
            if !changed && !revealed {
                break;
            }
        }
    }

    pub fn send_key(&mut self, code: KeyCode, modifiers: KeyModifiers) {
        let key = KeyEvent::new(code, modifiers);
        self.manager.dispatch(key);
    }

    pub fn type_char(&mut self, c: char) {
        self.send_key(KeyCode::Char(c), KeyModifiers::NONE);
    }

    pub fn type_text(&mut self, text: &str) {
        for c in text.chars() {
            if c == '\n' {
                self.send_key(KeyCode::Enter, KeyModifiers::NONE);
            } else if c == '\t' {
                self.send_key(KeyCode::Tab, KeyModifiers::NONE);
            } else {
                self.type_char(c);
            }
        }
    }

    pub fn click_at(&mut self, x: u16, y: u16) {
        self.manager.pointer_down_at(&self.hits, x, y);
    }

    pub fn scroll(&mut self, x: u16, y: u16, dx: i16, dy: i16) {
        if let Some(target) = self.hits.scroll_target(x, y) {
            self.manager.scroll(target, dx, dy);
        }
    }

    pub fn focused_id(&self) -> Option<FocusId> {
        self.manager.focused()
    }

    pub fn line(&self, y: u16) -> String {
        (0..self.width)
            .map(|x| self.buffer.get(x, y).map(|c| c.ch).unwrap_or(' '))
            .collect::<String>()
            .trim_end()
            .to_string()
    }

    pub fn cursor(&self) -> Option<(u16, u16)> {
        self.buffer.cursor()
    }

    pub fn cursor_style(&self) -> Option<CursorStyle> {
        self.buffer.cursor_style()
    }

    pub fn cell_fg(&self, x: u16, y: u16) -> Option<Color> {
        self.buffer.get(x, y).and_then(|c| c.fg)
    }
}

#[test]
fn test_live_preview_sync_per_keystroke() {
    let mut term = TestTerminal::new(100, 10);
    let (code, set_code) = create_signal(String::new());

    let build = move || {
        view! {
            <View flex_direction=FlexDirection::Row width={Dimension::Percent(1.0)} height={Dimension::Percent(1.0)} gap=2>
                <CodeEditor
                    value={code.get()} on_value_change={move |v: String| set_code.set(v)}
                    language="rs"
                    width=45
                    height=8
                    show_line_numbers=false
                    autofocus=true
                />
                <CodePreview
                    value={code.get()}
                    language="rs"
                    width=45
                    height=8
                    show_line_numbers=false
                />
            </View>
        }
    };

    term.draw(&build);

    // Type character 'f'
    term.type_char('f');
    term.draw(&build);
    assert_eq!(code.get(), "f");

    // Both editor and preview screen lines should now contain 'f'
    let line1 = term.line(1);
    assert!(
        line1.contains('f'),
        "Line 1 should contain typed character 'f' in both panes, got: {:?}",
        line1
    );

    // Type remaining characters of "fn main() {"
    term.type_text("n main() {");
    term.draw(&build);
    assert_eq!(code.get(), "fn main() {");

    let screen_line = term.line(1);
    assert!(
        screen_line.contains("fn main() {"),
        "Expected screen to show complete function signature, got: {:?}",
        screen_line
    );
}

#[test]
fn test_visual_code_editor_typing_and_cursor() {
    let mut term = TestTerminal::new(60, 10);
    let (code, set_code) = create_signal(String::new());

    let build = move || {
        view! {
            <CodeEditor
                value={code.get()} on_value_change={move |v: String| set_code.set(v)}
                language="rs"
                width=50
                height=8
                show_line_numbers=true
                autofocus=true
            />
        }
    };

    // 1. Initial draw - editor should be autofocus-active with line 1 gutter
    term.draw(&build);
    let l1 = term.line(1);
    assert!(
        l1.contains("1 │"),
        "Line 1 should contain gutter '1 │', but was: {:?}",
        l1
    );

    let initial_cursor = term.cursor();
    assert!(
        initial_cursor.is_some(),
        "Cursor should be visible when focused"
    );

    // 2. Type "fn main() {"
    term.type_text("fn main() {");
    term.draw(&build);

    let l1_after_typing = term.line(1);
    assert!(
        l1_after_typing.contains("1 │ fn main() {"),
        "Expected '1 │ fn main() {{', got: {:?}",
        l1_after_typing
    );

    // 3. Hit Enter -> should auto-indent 4 spaces on line 2
    term.send_key(KeyCode::Enter, KeyModifiers::NONE);
    term.draw(&build);

    let l2 = term.line(2);
    assert!(
        l2.contains("2 │     "),
        "Line 2 should auto-indent with 4 spaces after '{{', got: {:?}",
        l2
    );

    // 4. Type "let x = 42;" on line 2
    term.type_text("let x = 42;");
    term.draw(&build);

    let l2_typed = term.line(2);
    assert!(
        l2_typed.contains("2 │     let x = 42;"),
        "Line 2 should contain code, got: {:?}",
        l2_typed
    );

    // 5. Test Backspace navigation
    term.send_key(KeyCode::Backspace, KeyModifiers::NONE); // deletes ';'
    term.draw(&build);
    assert!(term.line(2).contains("2 │     let x = 42"));

    // 6. Test Undo (Ctrl+Z)
    term.send_key(KeyCode::Char('z'), KeyModifiers::CONTROL);
    term.draw(&build);
    assert!(
        term.line(2).contains("2 │     let x = 42;"),
        "Undo should restore ';'"
    );

    // 7. Verify the reactive signal contains the multiline string
    assert_eq!(code.get(), "fn main() {\n    let x = 42;");
}

#[test]
fn test_visual_cursor_movement_and_click() {
    let mut term = TestTerminal::new(60, 10);
    let (code, set_code) = create_signal(String::from("line one\nline two"));

    let build = move || {
        view! {
            <CodeEditor
                value={code.get()} on_value_change={move |v: String| set_code.set(v)}
                language="rs"
                width=50
                height=8
                show_line_numbers=true
                autofocus=true
            />
        }
    };

    term.draw(&build);

    // Move Up
    term.send_key(KeyCode::Up, KeyModifiers::NONE);
    term.draw(&build);

    // Home -> should move cursor to start of line
    term.send_key(KeyCode::Home, KeyModifiers::NONE);
    term.draw(&build);

    let (_cx, cy) = term.cursor().expect("Cursor should be set");
    assert_eq!(cy, 1, "Cursor should be on line row 1");

    // Type character at start of line
    term.type_char('#');
    term.draw(&build);

    assert!(term.line(1).contains("1 │ #line one"));
    assert_eq!(code.get(), "#line one\nline two");
}

#[test]
fn test_syntax_highlighting_colors_on_screen() {
    let mut term = TestTerminal::new(60, 10);
    let (code, set_code) = create_signal(String::from("fn test()"));

    let build = move || {
        view! {
            <CodeEditor
                value={code.get()} on_value_change={move |v: String| set_code.set(v)}
                language="rs"
                width=50
                height=8
                show_line_numbers=true
                autofocus=true
            />
        }
    };

    term.draw(&build);

    // Find the position of 'f' in "fn" on row 1
    let line1 = term.line(1);
    let fn_offset = line1
        .find("fn")
        .expect("'fn' must be present in screen line");

    let fn_color = term.cell_fg(fn_offset as u16, 1);
    assert!(
        fn_color.is_some(),
        "'fn' keyword must have syntax highlight color"
    );

    let Color::Rgb { r, g, b } = fn_color.unwrap() else {
        panic!("Expected RGB syntax color from TextMate theme");
    };
    assert!(r > 0 || g > 0 || b > 0);
}

#[test]
fn test_editor_height_fills_container() {
    let mut term = TestTerminal::new(80, 30);
    let lines = (1..=25)
        .map(|i| format!("println!(\"Line number {}\");", i))
        .collect::<Vec<_>>()
        .join("\n");
    let (code, set_code) = create_signal(lines);

    let build = move || {
        view! {
            <CodeEditor
                value={code.get()} on_value_change={move |v: String| set_code.set(v)}
                language="rs"
                width={Dimension::Percent(1.0)}
                height={Dimension::Percent(1.0)}
                show_line_numbers=true
                autofocus=true
            />
        }
    };

    term.draw(&build);

    // Verify that line 20 is rendered on screen (proving container fills beyond 12 lines)
    let mut found_line_20 = false;
    for y in 0..30 {
        if term.line(y).contains("Line number 20") {
            found_line_20 = true;
            break;
        }
    }
    assert!(
        found_line_20,
        "Line 20 must be visible on screen when height=Dimension::Percent(1.0) in a 30-row terminal"
    );
}

#[test]
fn test_code_preview_focus_and_keyboard_scroll() {
    let mut term = TestTerminal::new(100, 20);
    let editor_id = FocusId::new();
    let preview_id = FocusId::new();
    let content = (1..=30)
        .map(|i| format!("code line {}", i))
        .collect::<Vec<_>>()
        .join("\n");
    let (code, set_code) = create_signal(content);

    let build = move || {
        view! {
            <View flex_direction=FlexDirection::Row width={Dimension::Percent(1.0)} height={Dimension::Percent(1.0)} gap=1>
                <CodeEditor
                    id=editor_id
                    value={code.get()} on_value_change={move |v: String| set_code.set(v)}
                    width={Dimension::Percent(0.5)}
                    height={Dimension::Percent(1.0)}
                    autofocus=true
                />
                <CodePreview
                    id=preview_id
                    value={code.get()}
                    width={Dimension::Percent(0.5)}
                    height={Dimension::Percent(1.0)}
                />
            </View>
        }
    };

    term.draw(&build);

    // Initial state: editor is focused by autofocus
    assert_eq!(term.focused_id(), Some(editor_id));

    // Preview has single border character at top-left: column 51, row 0 (50 editor + 1 gap)
    let preview_tl_char = term.buffer.get(51, 0).map(|c| c.ch);
    assert_eq!(
        preview_tl_char,
        Some('┌'),
        "Unfocused preview should have single border"
    );

    // Click inside the preview container (column 60, row 5)
    term.click_at(60, 5);
    term.draw(&build);

    // Now preview must be focused!
    assert_eq!(
        term.focused_id(),
        Some(preview_id),
        "Clicking inside preview container should focus it"
    );

    // Focused preview has double border character at top-left
    let preview_tl_char_focused = term.buffer.get(51, 0).map(|c| c.ch);
    assert_eq!(
        preview_tl_char_focused,
        Some('╔'),
        "Focused preview should have double border"
    );

    // Test keyboard scrolling in preview
    assert_eq!(get_scroll(preview_id), (0, 0));
    term.send_key(KeyCode::Down, KeyModifiers::NONE);
    assert_eq!(
        get_scroll(preview_id).1,
        1,
        "Down arrow on focused preview should scroll down"
    );

    // Test Tab switches focus back to editor
    term.send_key(KeyCode::Tab, KeyModifiers::NONE);
    term.draw(&build);
    assert_eq!(
        term.focused_id(),
        Some(editor_id),
        "Tab should cycle focus back to editor"
    );
}

#[test]
fn test_code_editor_cursor_goes_beyond_20_lines() {
    // 40 rows high terminal
    let mut term = TestTerminal::new(80, 40);
    let lines = (1..=30)
        .map(|i| format!("println!(\"Line {}\");", i))
        .collect::<Vec<_>>()
        .join("\n");
    let (code, set_code) = create_signal(lines);

    let build = move || {
        view! {
            <CodeEditor
                value={code.get()} on_value_change={move |v: String| set_code.set(v)}
                language="rs"
                width={Dimension::Percent(1.0)}
                height={Dimension::Percent(1.0)}
                show_line_numbers=true
                autofocus=true
            />
        }
    };

    term.draw(&build);

    // Initial cursor: begins at line 1 (screen y = 1 inside border)
    let initial_cursor = term.cursor().expect("Cursor should be present");
    assert_eq!(
        initial_cursor.1, 1,
        "Cursor row should be at row 1 on initial load"
    );

    // Move down 25 times (well beyond line 20)
    for _ in 0..25 {
        term.send_key(KeyCode::Down, KeyModifiers::NONE);
    }
    term.draw(&build);
    let moved_cursor = term.cursor().expect("Cursor should be valid");
    assert_eq!(
        moved_cursor.1, 26,
        "Cursor must be at row 26 (line 26), well beyond line 20"
    );

    // Move down 5 more times (to line 30)
    for _ in 0..5 {
        term.send_key(KeyCode::Down, KeyModifiers::NONE);
    }
    term.draw(&build);
    let line30_cursor = term.cursor().expect("Cursor should be valid");
    assert_eq!(
        line30_cursor.1, 30,
        "Cursor must be at row 30 (line 30), well beyond line 20"
    );

    // Press Enter to insert lines beyond line 30
    for _ in 0..5 {
        term.send_key(KeyCode::Enter, KeyModifiers::NONE);
    }
    term.draw(&build);
    let after_enter_cursor = term.cursor().expect("Cursor should follow Enter");
    assert_eq!(
        after_enter_cursor.1, 35,
        "Cursor must advance past line 30 to row 35"
    );
}

#[test]
fn test_code_editor_cursor_style_choice_block_and_line() {
    let mut term = TestTerminal::new(60, 20);
    let (style, set_style) = create_signal(CursorStyle::Line);

    let build = move || {
        view! {
            <CodeEditor
                language="rs"
                width=50
                height=15
                cursor_style={style.get()}
                autofocus=true
            />
        }
    };

    // Draw with Line cursor
    term.draw(&build);
    assert_eq!(
        term.cursor_style(),
        Some(CursorStyle::Line),
        "Buffer should have Line cursor style"
    );

    // Switch to Block cursor
    set_style.set(CursorStyle::Block);
    term.draw(&build);
    assert_eq!(
        term.cursor_style(),
        Some(CursorStyle::Block),
        "Buffer should reflect Block cursor style when toggled"
    );

    // Switch to Underline cursor
    set_style.set(CursorStyle::Underline);
    term.draw(&build);
    assert_eq!(
        term.cursor_style(),
        Some(CursorStyle::Underline),
        "Buffer should reflect Underline cursor style when toggled"
    );
}

#[test]
fn test_code_editor_mouse_scrolling_and_cursor_movement() {
    let mut term = TestTerminal::new(80, 20);
    let editor_id = FocusId::new();
    let lines = (1..=30)
        .map(|i| format!("fn item_{}() {{ println!(\"item {}\"); }}", i, i))
        .collect::<Vec<_>>()
        .join("\n");
    let (code, set_code) = create_signal(lines);

    let build = move || {
        view! {
            <CodeEditor
                id=editor_id
                value={code.get()} on_value_change={move |v: String| set_code.set(v)}
                language="rs"
                width={Dimension::Percent(1.0)}
                height={Dimension::Percent(1.0)}
                show_line_numbers=true
                autofocus=true
            />
        }
    };

    term.draw(&build);

    // Initial state: cursor is on line 1, row 1
    assert_eq!(term.cursor().map(|c| c.1), Some(1));

    // 1. Move cursor with mouse click
    // Click at screen column 15, row 5 (corresponds to line 5)
    term.click_at(15, 5);
    term.draw(&build);
    assert_eq!(
        term.cursor().map(|c| c.1),
        Some(5),
        "Mouse click at row 5 must move cursor to row 5"
    );

    // 2. Mouse wheel scroll down via container (dy = 4)
    term.scroll(15, 5, 0, 4);
    term.draw(&build);

    // Verify container scroll offset updated
    let scroll_pos = get_scroll(editor_id);
    assert_eq!(
        scroll_pos.1, 4,
        "Container scroll must be 4 after mouse wheel scroll down"
    );

    // First line on screen should now be line 5
    let top_line = term.line(1);
    assert!(
        top_line.contains("5 │"),
        "Top line should be line 5 after scrolling 4 lines down, got: {:?}",
        top_line
    );

    // 3. Move cursor with mouse click while container is scrolled
    // Click at row 3 (which now displays line 7 because scroll_y is 4: row 3 + scroll 4 = line 7)
    term.click_at(15, 3);
    term.draw(&build);
    assert_eq!(
        term.cursor().map(|c| c.1),
        Some(3),
        "Cursor should be positioned at row 3 where clicked"
    );

    // 4. Mouse wheel scroll back up via container (dy = -4)
    term.scroll(15, 5, 0, -4);
    term.draw(&build);
    assert_eq!(
        get_scroll(editor_id).1,
        0,
        "Container scroll should return to 0"
    );
    assert!(
        term.line(1).contains("1 │"),
        "Top line should be line 1 again"
    );
}
