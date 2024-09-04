use crate::util::Direction;
use crossterm::{cursor, terminal};
use std::io;

#[derive(Clone, Copy, Default, Debug)]
pub struct Position {
    pub x: u16,
    pub y: u16,
}

#[derive(Clone, Copy, Default, Debug)]
pub struct Size {
    pub width: u16,
    pub height: u16,
}

pub fn size() -> io::Result<Size> {
    terminal::size().map(|(width, height)| Size { width, height })
}

pub fn queue(command: impl crossterm::Command) -> io::Result<()> {
    crossterm::queue!(io::stdout(), command)
}

pub fn flush() -> io::Result<()> {
    io::Write::flush(&mut io::stdout())
}

pub fn begin() -> io::Result<()> {
    terminal::enable_raw_mode()?;
    queue(terminal::EnterAlternateScreen)?;
    queue(terminal::DisableLineWrap)?;
    Ok(())
}

pub fn end() -> io::Result<()> {
    terminal::disable_raw_mode()?;
    queue(terminal::LeaveAlternateScreen)?;
    queue(terminal::EnableLineWrap)?;
    queue(cursor::Show)?;
    Ok(())
}

pub fn clear() -> io::Result<()> {
    queue(terminal::Clear(terminal::ClearType::All))
}

pub fn clear_line() -> io::Result<()> {
    queue(terminal::Clear(terminal::ClearType::CurrentLine))
}

pub fn set_cursor(Position { x, y }: Position) -> io::Result<()> {
    queue(cursor::MoveTo(x, y))
}

impl Position {
    pub fn move_toward(self, direction: Direction) -> Position {
        match direction {
            Direction::Up => Position { y: self.y.saturating_sub(1), ..self },
            Direction::Down => Position { y: self.y.saturating_add(1), ..self },
            Direction::Left => Position { x: self.x.saturating_sub(1), ..self },
            Direction::Right => Position { x: self.x.saturating_add(1), ..self },
        }
    }
    pub fn offset_x(self, x: u16) -> Position {
        Position { x: self.x + x, y: self.y }
    }
    pub fn offset_y(self, y: u16) -> Position {
        Position { x: self.x, y: self.y + y }
    }
    pub fn offset(self, other: Position) -> Position {
        self.offset_x(other.x).offset_y(other.y)
    }
}
