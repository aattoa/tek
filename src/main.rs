mod editor;
mod indexvec;
mod settings;
mod terminal;
mod text;
mod ui;
mod util;

fn main() -> std::io::Result<()> {
    terminal::begin()?;
    let result = ui::UI::new(terminal::size()?).run();
    terminal::end()?;
    result
}
