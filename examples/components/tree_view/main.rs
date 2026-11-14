use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use sveltui::prelude::*;
use sveltui_components::*;

#[cfg(test)]
mod tree_view_tests;

/// A file or directory, read from disk only when its parent is opened.
#[derive(Clone)]
struct Entry {
    path: PathBuf,
    name: String,
    is_dir: bool,
}

impl Entry {
    fn new(path: PathBuf) -> Self {
        let name = path
            .file_name()
            .map_or_else(String::new, |n| n.to_string_lossy().into_owned());
        let is_dir = path.is_dir();
        Self { path, name, is_dir }
    }

    /// Directories first, then files, each by name.
    fn read_dir(&self) -> Vec<Entry> {
        let mut entries: Vec<Entry> = std::fs::read_dir(&self.path)
            .map(|dir| dir.flatten().map(|e| Entry::new(e.path())).collect())
            .unwrap_or_default();
        entries.sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then_with(|| a.name.cmp(&b.name)));
        entries
    }
}

fn crates_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../crates")
}

#[component]
fn TreeViewDemo() -> impl IntoNode {
    let roots = use_hook(|| Rc::new(Entry::new(crates_dir()).read_dir()));
    let opened = state(String::from("Enter opens a file."));
    let setting = state(HashSet::<String>::new());
    let model = TreeModel::new(
        |e: &Entry| e.path.to_string_lossy().into_owned(),
        |e: &Entry| e.name.clone(),
        |e: &Entry| e.read_dir(),
    )
    .is_branch(|e: &Entry| e.is_dir);
    let open_file = move |key: String| {
        let name = Path::new(&key)
            .file_name()
            .map_or_else(String::new, |n| n.to_string_lossy().into_owned());
        opened.set(format!("Opened {name}."));
    };
    let picked = setting
        .get()
        .into_iter()
        .next()
        .unwrap_or_else(|| "nothing".to_owned());

    view! {
        <View width={Dimension::Percent(1.0)} height={Dimension::Percent(1.0)} padding=1 gap=1>
            <View flex_direction=FlexDirection::Row gap=2 flex_grow=1.0 flex_basis=0>
                <View flex_grow=1.0 flex_basis=0>
                    <TreeView roots={GridRows::Shared(roots.clone())} model={model.clone()}
                        title="crates" height={Dimension::Percent(1.0)} on_activate=open_file />
                </View>
                <View flex_grow=1.0 flex_basis=0>
                    <TreeView::Root title="Settings" height={Dimension::Percent(1.0)}
                        bind:selected={setting}>
                        <TreeView::Branch value="editor" label="Editor">
                            <TreeView::Item value="editor.font" label="Font" />
                            <TreeView::Item value="editor.tabs" label="Tabs" />
                            <TreeView::Branch value="editor.cursor" label="Cursor">
                                <TreeView::Item value="editor.cursor.blink" label="Blink" />
                                <TreeView::Item value="editor.cursor.shape" label="Shape" />
                            </TreeView::Branch>
                        </TreeView::Branch>
                        <TreeView::Branch value="terminal" label="Terminal">
                            <TreeView::Item value="terminal.shell" label="Shell" />
                        </TreeView::Branch>
                        <TreeView::Item value="keys" label="Keybindings" />
                    </TreeView::Root>
                </View>
            </View>
            <TextSpan foreground_color=Color::DarkGrey>
                {format!("{} · Setting: {picked}", opened.get())}
            </TextSpan>
            <TextSpan foreground_color=Color::DarkGrey>
                "Tab switches tree · ↑↓ move · → opens · ← closes · Space selects · type to jump · q quits"
            </TextSpan>
        </View>
    }
}

fn main() -> std::io::Result<()> {
    App::new(|| view! { <TreeViewDemo /> })
        .on_key(sveltui::app::exit_shortcuts)
        .run()
}
