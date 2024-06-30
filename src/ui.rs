use crate::editor;
use crate::terminal::{self, Position};
use crate::util::Direction;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::{cursor, style};
use std::io;

pub struct UI {
    editor: editor::Editor,
    size: terminal::Size,
    status: Option<String>,
    quit: bool,
}

fn draw_status_line(ui: &UI) -> io::Result<()> {
    terminal::set_cursor(Position { x: 0, y: ui.size.height })?;
    terminal::queue(style::SetBackgroundColor(style::Color::DarkGrey))?;
    terminal::clear_line()?;

    print!("-- {:?} --", ui.editor.mode);
    if let Some(string) = &ui.status {
        print!(" {string}");
    }
    if let Some(cursor) = ui.editor.focus.map(|focus| ui.editor.windows[focus].cursor) {
        print!(" {},{}", cursor.x + 1, cursor.y + 1);
    }

    terminal::queue(style::SetBackgroundColor(style::Color::Reset))?;
    Ok(())
}

fn line_view(line: &str, view: editor::View) -> &str {
    let from = view.offset as usize;
    let to = from + view.width as usize;
    line.get(from..to).unwrap_or(line)
}

fn draw_view(ui: &UI, view: editor::View, position: Position) -> io::Result<()> {
    let text: String = ui.editor.buffers[view.buffer].text().unwrap().gather();
    let lines: Vec<&str> = text.lines().collect();
    let number_width = lines.len().to_string().len();
    for (index, &line) in lines.iter().enumerate() {
        terminal::set_cursor(Position { x: position.x, y: position.y + index as u16 })?;
        terminal::queue(style::SetForegroundColor(style::Color::DarkGrey))?;
        terminal::queue(style::SetAttribute(style::Attribute::Bold))?;
        print!("{:number_width$}", index + 1);
        terminal::queue(style::SetAttribute(style::Attribute::Reset))?;
        terminal::queue(style::SetForegroundColor(style::Color::Reset))?;
        print!(" {}", line_view(line, view));
    }
    Ok(())
}

fn draw_windows(ui: &UI) -> io::Result<()> {
    for window in &ui.editor.windows.underlying {
        if let Some(view) = window.view {
            draw_view(ui, view, window.position)?;
        }
    }
    draw_status_line(ui)?;
    Ok(())
}

fn draw_cursor(ui: &UI) -> io::Result<()> {
    if let Some(window) = ui.editor.focus {
        let window = &ui.editor.windows[window];
        terminal::set_cursor(window.position.offset(window.cursor))?;
        terminal::queue(cursor::Show)?;
    }
    Ok(())
}

fn draw(ui: &UI) -> io::Result<()> {
    terminal::queue(cursor::Hide)?;
    terminal::clear()?;
    draw_windows(ui)?;
    draw_cursor(ui)?;
    terminal::flush()?;
    Ok(())
}

fn handle_key(ui: &mut UI, key: KeyEvent) -> io::Result<()> {
    if key.kind != KeyEventKind::Press {
        return Ok(());
    }

    match ui.editor.mode {
        editor::Mode::Normal => match key.code {
            KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => {
                ui.quit = true;
            }
            KeyCode::Char('w') if key.modifiers == KeyModifiers::CONTROL => {
                ui.editor.mode = editor::Mode::Window;
            }
            KeyCode::Char(character) => match character {
                'f' => {
                    let window = ui.create_window();
                    ui.editor.edit("test.txt".into(), window)?;
                }
                'h' => ui.editor.move_cursor(Direction::Left),
                'j' => ui.editor.move_cursor(Direction::Down),
                'k' => ui.editor.move_cursor(Direction::Up),
                'l' => ui.editor.move_cursor(Direction::Right),
                'i' => ui.editor.mode = editor::Mode::Insert,
                _ => {}
            },
            _ => {}
        },
        editor::Mode::Window => match key.code {
            KeyCode::Esc => ui.editor.mode = editor::Mode::Normal,
            KeyCode::Char(character) => match character {
                's' => ui.editor.horizontal_split_window(),
                'v' => ui.editor.vertical_split_window(),
                'w' => ui.editor.rotate_focus_forward(),
                'W' => ui.editor.rotate_focus_backward(),
                _ => {}
            },
            _ => {}
        },
        editor::Mode::Insert => match key.code {
            KeyCode::Esc => ui.editor.mode = editor::Mode::Normal,
            KeyCode::Char(character) => ui.status = Some(format!("got '{character}'")),
            _ => {}
        },
        _ => {}
    }

    Ok(())
}

fn handle_event(ui: &mut UI, event: Event) -> io::Result<()> {
    match event {
        Event::Key(event) => {
            handle_key(ui, event)?;
        }
        Event::Resize(width, height) => {
            ui.size = terminal::Size { width, height };
        }
        _ => {}
    }
    Ok(())
}

impl UI {
    pub fn new(size: terminal::Size) -> UI {
        UI {
            editor: editor::Editor::default(),
            size,
            status: None,
            quit: false,
        }
    }

    pub fn run(&mut self) -> io::Result<()> {
        while !self.quit {
            draw(self)?;
            handle_event(self, event::read()?)?;
        }
        Ok(())
    }

    pub fn create_window(&mut self) -> editor::WindowID {
        self.editor.windows.push(editor::Window {
            position: Default::default(),
            cursor: Default::default(),
            size: self.size,
            view: None,
        })
    }
}
