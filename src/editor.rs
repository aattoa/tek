use crate::indexvec::IndexVec;
use crate::settings;
use crate::terminal::{Position, Size};
use crate::text::PieceTable;
use crate::util::Direction;
use std::io;
use std::path::PathBuf;
use std::time::SystemTime;

crate::define_index!(pub BufferID);
crate::define_index!(pub WindowID);

pub type BufferVec = IndexVec<Buffer, BufferID>;
pub type WindowVec = IndexVec<Window, WindowID>;

#[derive(Clone, Copy, Default, Debug)]
pub enum Mode {
    #[default]
    Normal,
    Visual,
    Insert,
    Window,
    CommandLine,
    OperatorPending,
}

pub struct FileInfo {
    pub path: PathBuf,
    pub time: SystemTime,
}

#[derive(Default)]
pub struct Buffer {
    pub text: PieceTable,
    pub file_info: Option<FileInfo>,
    pub settings: settings::BufferSettings,
}

#[derive(Clone, Copy, Debug)]
pub struct View {
    pub offset: u16,
    pub size: Size,
    pub buffer: BufferID,
}

#[derive(Clone, Copy, Debug)]
pub struct Window {
    pub position: Position,
    pub cursor: Position,
    pub size: Size,
    pub view: Option<View>,
    pub settings: settings::WindowSettings,
}

pub struct Editor {
    pub buffers: BufferVec,
    pub windows: WindowVec,
    pub focus: WindowID,
    pub mode: Mode,
    pub size: Size,
    pub settings: settings::EditorSettings,
    pub status: Option<String>,
}

impl FileInfo {
    pub fn new(path: PathBuf) -> std::io::Result<FileInfo> {
        std::fs::metadata(&path)?.modified().map(|time| FileInfo { path, time })
    }
}

impl Buffer {
    pub fn read(path: std::path::PathBuf) -> std::io::Result<Buffer> {
        let text = std::fs::read_to_string(&path)?.into();
        let file_info = Some(FileInfo::new(path)?);
        let settings = settings::BufferSettings::default();
        Ok(Buffer { text, file_info, settings })
    }
}

impl Window {
    pub fn keep_cursor_within_bounds(&mut self) {
        self.cursor.x = self.cursor.x.min(self.size.width.saturating_sub(1));
        self.cursor.y = self.cursor.y.min(self.size.height.saturating_sub(1));
    }
    pub fn contains_x(&self, x: u16) -> bool {
        (self.position.x <= x) && (x < self.position.x + self.size.width)
    }
    pub fn contains_y(&self, y: u16) -> bool {
        (self.position.y <= y) && (y < self.position.y + self.size.height)
    }
    pub fn contains(&self, position: Position) -> bool {
        self.contains_x(position.x) && self.contains_y(position.y)
    }
}

impl Editor {
    pub fn emit_message(&mut self, message: String) {
        self.status = Some(message);
    }

    pub fn new(size: Size) -> Editor {
        let mut windows = WindowVec::new();

        let default_window = windows.push(Window {
            position: Position::default(),
            cursor: Position::default(),
            size: Size { width: size.width, height: size.height - 1 },
            view: None,
            settings: settings::WindowSettings::default(),
        });

        Editor {
            buffers: BufferVec::new(),
            windows,
            focus: default_window,
            mode: Mode::Normal,
            size,
            settings: settings::EditorSettings::default(),
            status: None,
        }
    }

    pub fn window_ids(&self) -> impl Iterator<Item = WindowID> {
        (0..self.windows.len()).map(crate::indexvec::VecIndex::new)
    }

    // TODO: check if already open
    pub fn edit(&mut self, path: PathBuf) -> io::Result<()> {
        let buffer = self.buffers.push(Buffer::read(path)?);
        let window = &mut self.windows[self.focus];
        window.cursor = Position::default();
        let size = Size { width: window.size.width - 2, height: window.size.height - 2 };
        window.view = Some(View { offset: window.position.x + 1, size, buffer });
        Ok(())
    }

    pub fn move_cursor(&mut self, direction: Direction) {
        let window = &mut self.windows[self.focus];
        window.cursor = window.cursor.move_toward(direction);
        window.keep_cursor_within_bounds();
    }

    pub fn rotate_focus_forward(&mut self) {
        let index = 1 + crate::indexvec::VecIndex::get(self.focus);
        let index = if index == self.windows.len() { 0 } else { index };
        self.focus = crate::indexvec::VecIndex::new(index);
    }

    pub fn rotate_focus_backward(&mut self) {
        let index = crate::indexvec::VecIndex::get(self.focus);
        let index = if index == 0 { self.windows.len() } else { index };
        self.focus = crate::indexvec::VecIndex::new(index - 1);
    }

    fn run_cursor_focus_beam(&self, beam: impl Iterator<Item = Position>) -> Option<WindowID> {
        beam.flat_map(|position| {
            self.window_ids().filter(move |&window_id| {
                window_id != self.focus && self.windows[window_id].contains(position)
            })
        })
        .next()
    }

    fn send_cursor_focus_beam(&self, direction: Direction) -> Option<WindowID> {
        let window = &self.windows[self.focus];
        let cursor = window.position.offset(window.cursor);

        match direction {
            Direction::Up => {
                let range = 0..window.position.y;
                self.run_cursor_focus_beam(range.rev().map(|y| Position { x: cursor.x, y }))
            }
            Direction::Down => {
                let range = window.position.y..self.size.height;
                self.run_cursor_focus_beam(range.map(|y| Position { x: cursor.x, y }))
            }
            Direction::Left => {
                let range = 0..window.position.x;
                self.run_cursor_focus_beam(range.rev().map(|x| Position { x, y: cursor.y }))
            }
            Direction::Right => {
                let range = window.position.x + window.size.width..self.size.width;
                self.run_cursor_focus_beam(range.map(|x| Position { x, y: cursor.y }))
            }
        }
    }

    pub fn move_focus(&mut self, direction: Direction) {
        self.send_cursor_focus_beam(direction).inspect(|&id| self.focus = id);
    }

    pub fn vertical_split_window(&mut self) {
        let left = &mut self.windows[self.focus];
        if left.size.width < 4 {
            self.emit_message(String::from("The window is too small for a vertical split"));
            return;
        }
        let remainder = left.size.width % 2;
        left.size.width /= 2;
        left.keep_cursor_within_bounds();
        let mut right: Window = *left;
        right.size.width += remainder;
        right.position.x += left.size.width;
        self.windows.push(right);
    }

    pub fn horizontal_split_window(&mut self) {
        let above = &mut self.windows[self.focus];
        if above.size.height < 4 {
            self.emit_message(String::from("The window is too small for a horizontal split"));
            return;
        }
        let remainder = above.size.height % 2;
        above.size.height /= 2;
        above.keep_cursor_within_bounds();
        let mut below: Window = *above;
        below.size.height += remainder;
        below.position.y += above.size.height;
        self.windows.push(below);
    }
}
