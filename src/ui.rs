use crate::terminal;
use crossterm::{event, style};
use std::io;
use std::time::Duration;

pub struct UI {
    quit: bool,
    status: String,
    size: terminal::Size,
    cursor: terminal::Position,
}

fn draw_status_line(ui: &UI) -> io::Result<()> {
    terminal::queue(style::SetBackgroundColor(style::Color::DarkGrey))?;
    terminal::clear_line()?;
    print!("status: {}, {},{}", ui.status, ui.cursor.x + 1, ui.cursor.y + 1);
    terminal::queue(style::SetBackgroundColor(style::Color::Reset))?;
    Ok(())
}

fn draw(ui: &UI) -> io::Result<()> {
    terminal::reset_cursor()?;
    for i in 1..ui.size.height {
        print!("~ {i}\r\n");
    }
    draw_status_line(ui)?;
    Ok(())
}

fn refresh(ui: &UI) -> io::Result<()> {
    terminal::show_cursor(false)?;
    terminal::clear()?;
    draw(ui)?;
    terminal::move_cursor(ui.cursor)?;
    terminal::show_cursor(true)?;
    terminal::flush()?;
    Ok(())
}

fn handle_key(ui: &mut UI, key: event::KeyEvent) -> io::Result<()> {
    if key.kind != event::KeyEventKind::Press {
        return Ok(());
    }
    match key.code {
        event::KeyCode::Char('c') if key.modifiers == event::KeyModifiers::CONTROL => {
            ui.quit = true;
        }
        event::KeyCode::Char('h') => {
            ui.cursor.x = ui.cursor.x.saturating_sub(1);
        }
        event::KeyCode::Char('j') => {
            ui.cursor.y = ui.cursor.y.saturating_add(1);
        }
        event::KeyCode::Char('k') => {
            ui.cursor.y = ui.cursor.y.saturating_sub(1);
        }
        event::KeyCode::Char('l') => {
            ui.cursor.x = ui.cursor.x.saturating_add(1);
        }
        event::KeyCode::Char('G') => {
            ui.cursor.y = ui.size.height;
        }
        event::KeyCode::Char('g') => {
            ui.cursor.y = 0;
        }
        _ => {}
    }
    Ok(())
}

fn update_cursor(ui: &mut UI) {
    ui.cursor.x = ui.cursor.x.min(ui.size.width.saturating_sub(1));
    ui.cursor.y = ui.cursor.y.min(ui.size.height.saturating_sub(1));
}

impl UI {
    pub fn new(size: terminal::Size) -> UI {
        UI {
            status: String::from("default"),
            size,
            quit: false,
            cursor: terminal::Position { x: 0, y: 0 },
        }
    }

    pub fn run(&mut self) -> io::Result<()> {
        while !self.quit {
            update_cursor(self);
            refresh(self)?;
            if event::poll(Duration::from_secs(1))? {
                match event::read()? {
                    event::Event::Key(event) => {
                        handle_key(self, event)?;
                    }
                    event::Event::Resize(width, height) => {
                        self.size = terminal::Size { width, height };
                    }
                    ev => self.status = format!("event: {ev:?}"),
                }
            }
        }
        Ok(())
    }
}
