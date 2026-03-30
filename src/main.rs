mod editor;
use editor::Editor;

fn main() {
    if let Ok(mut editor) = Editor::new() {
        editor.run().unwrap();
    } else {
        println!("Error in the Editor");
    }

    println!("Goodbye.");
}
