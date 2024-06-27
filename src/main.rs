mod buffer;
mod editor;
mod terminal;
mod text;
mod ui;

fn main() -> std::io::Result<()> {
    terminal::begin()?;
    let result = ui::UI::new(terminal::size()?).run();
    terminal::end()?;
    result
}
