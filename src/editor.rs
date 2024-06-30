#![allow(dead_code)]

use crate::define_index;
use crate::indexvec::IndexVec;
use crate::terminal::{Position, Size};
use crate::text::PieceTable;
use crate::util::Direction;
use std::io;
use std::path::PathBuf;
use std::time::SystemTime;

define_index!(pub BufferID);
define_index!(pub WindowID);

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

pub enum Buffer {
    File {
        text: PieceTable,
        path: PathBuf,
        time: SystemTime,
    },
    New(PieceTable),
}

#[derive(Clone, Copy, Debug)]
pub struct View {
    pub offset: u16,
    pub width: u16,
    pub buffer: BufferID,
}

#[derive(Clone, Copy, Debug)]
pub struct Window {
    pub position: Position,
    pub cursor: Position,
    pub size: Size,
    pub view: Option<View>,
}

#[derive(Default)]
pub struct Editor {
    pub buffers: IndexVec<Buffer, BufferID>,
    pub windows: IndexVec<Window, WindowID>,
    pub focus: Option<WindowID>,
    pub mode: Mode,
}

impl Buffer {
    pub fn new() -> Buffer {
        Buffer::New(PieceTable::default())
    }

    pub fn read(path: std::path::PathBuf) -> std::io::Result<Buffer> {
        let time = std::fs::metadata(&path)?.modified()?;
        let text = std::fs::read_to_string(&path);
        text.map(PieceTable::from).map(|text| Buffer::File { text, path, time })
    }

    pub fn text(&self) -> Option<&PieceTable> {
        match self {
            Buffer::File { text, .. } | Buffer::New(text) => Some(text),
        }
    }
}

impl Window {
    pub fn keep_cursor_within_bounds(&mut self) {
        self.cursor.x = self.cursor.x.min(self.size.width.saturating_sub(1));
        self.cursor.y = self.cursor.y.min(self.size.height.saturating_sub(1));
    }
}

impl Editor {
    // TODO: check if already open
    pub fn edit(&mut self, path: PathBuf, window_id: WindowID) -> io::Result<()> {
        let buffer = self.buffers.push(Buffer::read(path)?);
        let window = &mut self.windows[window_id];
        window.cursor = Position::default();
        window.view = Some(View { offset: window.position.x, width: window.size.width, buffer });
        self.focus = Some(window_id);
        Ok(())
    }

    pub fn move_cursor(&mut self, direction: Direction) {
        if let Some(window) = self.focus {
            let window = &mut self.windows[window];
            window.cursor = window.cursor.move_toward(direction);
            window.keep_cursor_within_bounds();
        }
    }

    pub fn rotate_focus_forward(&mut self) {
        if let Some(window) = self.focus {
            let index = 1 + crate::indexvec::VecIndex::get(window);
            let index = if index == self.windows.len() { 0 } else { index };
            self.focus = Some(crate::indexvec::VecIndex::new(index));
        }
    }

    pub fn rotate_focus_backward(&mut self) {
        if let Some(window) = self.focus {
            let index = crate::indexvec::VecIndex::get(window);
            let index = if index == 0 { self.windows.len() } else { index };
            self.focus = Some(crate::indexvec::VecIndex::new(index - 1));
        }
    }

    pub fn vertical_split_window(&mut self) {
        if let Some(window) = self.focus {
            let above = &mut self.windows[window];
            above.size.width /= 2;
            above.keep_cursor_within_bounds();
            let mut below: Window = *above;
            below.position.x += above.size.width;
            self.windows.push(below);
        }
    }

    pub fn horizontal_split_window(&mut self) {
        if let Some(window) = self.focus {
            let above = &mut self.windows[window];
            above.size.height /= 2;
            above.keep_cursor_within_bounds();
            let mut below: Window = *above;
            below.position.y += above.size.height;
            self.windows.push(below);
        }
    }
}
