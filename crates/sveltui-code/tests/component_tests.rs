use sveltui::element::{IntoNode, VNode};
use sveltui::layout::Dimension;
use sveltui::reactive::{create_signal, state};
use sveltui::view;
use sveltui_code::prelude::*;

#[test]
fn test_code_preview_renders() {
    let code = "fn test() -> bool {\n    true\n}";

    let node = view! {
        <CodePreview
            code={code.to_string()}
            language="rs"
            show_line_numbers=true
            width={Dimension::Length(40.0)}
            height={Dimension::Length(6.0)}
            title="test.rs"
        />
    };

    let vnode = node.into_node();
    // The pressable attaches to the framed view instead of wrapping it.
    match vnode {
        VNode::View(v) => {
            assert_eq!(v.pressables.len(), 1);
            assert_eq!(
                v.visual.as_ref().and_then(|vis| vis.title.as_deref()),
                Some("test.rs")
            );
        }
        _ => panic!("Expected CodePreview root to be a pressable View"),
    }
}

#[test]
fn test_code_editor_renders() {
    let (code_read, _code_write) = create_signal(String::from("let x = 10;"));

    let node = view! {
        <CodeEditor
            value={code_read.get()}
            language="rs"
            show_line_numbers=true
            width={Dimension::Length(50.0)}
            height={Dimension::Length(10.0)}
            title="Editor"
        />
    };

    let vnode = node.into_node();
    match vnode {
        VNode::View(v) => {
            assert!(v.pressables.iter().any(|p| p.on_key.is_some()));
            assert_eq!(
                v.visual.as_ref().and_then(|vis| vis.title.as_deref()),
                Some("Editor")
            );
        }
        _ => panic!("Expected CodeEditor root to be a pressable View"),
    }
}

#[test]
fn test_code_binding_from_state() {
    let code_state = state(String::from("print('hello')"));

    let node = view! {
        <CodeEditor
            bind:value=code_state
            language="py"
            show_line_numbers=false
        />
    };

    let vnode = node.into_node();
    assert!(matches!(vnode, VNode::View(v) if !v.pressables.is_empty()));
}
