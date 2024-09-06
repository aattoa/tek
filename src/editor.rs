use crate::indexvec::IndexVec;
use crate::terminal::{Position, Size};
use crate::{settings, text, util};
use std::io;
use std::path::PathBuf;
use std::time::SystemTime;

crate::define_index!(pub BufferID);
crate::define_index!(pub WindowID);

pub type BufferVec = IndexVec<Buffer, BufferID>;
pub type WindowVec = IndexVec<Window, WindowID>;

#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum Mode {
    #[default]
    Normal,
    Insert,
    Window,
    CommandLine,
}

pub struct FileInfo {
    pub path: PathBuf,
    pub time: SystemTime,
}

#[derive(Default)]
pub struct Buffer {
    pub text: text::PieceTable,
    pub file_info: Option<FileInfo>,
    pub settings: settings::BufferSettings,
    pub windows: Vec<WindowID>,
}

#[derive(Clone, Copy, Debug)]
pub struct View {
    pub offset: u16,
    pub size: Size,
    pub buffer: BufferID,
}

#[derive(Clone, Copy, Default, Debug)]
pub struct Window {
    pub position: Position,
    pub cursor: Position,
    pub size: Size,
    pub view: Option<View>,
    pub settings: settings::WindowSettings,
    pub is_open: bool,
    pub redraw: bool,
}

pub struct Tab {
    pub open_windows: Vec<WindowID>,
    pub window_focus: WindowID,
}

pub struct Editor {
    pub buffers: BufferVec,
    pub windows: WindowVec,
    pub tabs: Vec<Tab>,
    pub status: Option<String>,
    pub mode: Mode,
    pub size: Size,
    pub settings: settings::EditorSettings,
    pub current_tab: usize,
}

impl FileInfo {
    pub fn new(path: PathBuf) -> std::io::Result<FileInfo> {
        std::fs::metadata(&path)?.modified().map(|time| FileInfo { path, time })
    }
}

impl Buffer {
    pub fn read(path: std::path::PathBuf) -> std::io::Result<Buffer> {
        Ok(Buffer {
            text: std::fs::read_to_string(&path)?.into(),
            file_info: Some(FileInfo::new(path)?),
            settings: settings::BufferSettings::default(),
            windows: Vec::new(),
        })
    }
}

impl Window {
    pub fn new(position: Position, size: Size) -> Window {
        Window {
            position,
            cursor: Position::default(),
            size,
            view: None,
            settings: settings::WindowSettings::default(),
            is_open: false,
            redraw: true,
        }
    }
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
    pub fn new_tab(&mut self) -> Tab {
        let default_window_id = self.windows.push(Window {
            size: Size { height: self.size.height - 1, ..self.size },
            is_open: true,
            redraw: true,
            ..Window::default()
        });
        Tab {
            open_windows: vec![default_window_id],
            window_focus: default_window_id,
        }
    }

    pub fn new(size: Size) -> Editor {
        let mut editor = Editor {
            buffers: BufferVec::new(),
            windows: WindowVec::new(),
            tabs: Vec::new(),
            mode: Mode::Normal,
            size,
            settings: settings::EditorSettings::default(),
            status: None,
            current_tab: 0,
        };
        let tab = editor.new_tab();
        editor.tabs.push(tab);
        editor
    }

    pub fn emit_message(&mut self, message: String) {
        self.status = Some(message);
    }

    pub fn force_redraw(&mut self) {
        for window in &mut self.windows.underlying {
            window.redraw = true;
        }
    }

    pub fn window_focus(&self) -> WindowID {
        self.tabs[self.current_tab].window_focus
    }

    pub fn window_ids(&self) -> impl Iterator<Item = WindowID> {
        (0..self.windows.len()).map(crate::indexvec::VecIndex::new)
    }

    pub fn new_window(&mut self) -> WindowID {
        self.window_ids()
            .find(|&id| !self.windows[id].is_open)
            .unwrap_or_else(|| self.windows.push(Window::default()))
    }

    fn set_window_focus(&mut self, new_focus: WindowID) {
        let tab = &mut self.tabs[self.current_tab];
        self.windows[tab.window_focus].redraw = true;
        self.windows[new_focus].redraw = true;
        tab.window_focus = new_focus;
    }

    fn set_current_tab(&mut self, tab: usize) {
        assert!(tab < self.tabs.len());
        self.current_tab = tab;
        self.force_redraw();
    }

    // TODO: check if already open
    pub fn edit(&mut self, path: PathBuf) -> io::Result<()> {
        let buffer = self.buffers.push(Buffer::read(path)?);
        let window = &mut self.windows[self.tabs[self.current_tab].window_focus];
        let size = Size { width: window.size.width - 2, height: window.size.height - 2 };
        window.view = Some(View { offset: window.position.x + 1, size, buffer });
        window.cursor = Position::default();
        window.redraw = true;
        Ok(())
    }

    pub fn tab_open(&mut self) {
        let tab = self.new_tab();
        self.tabs.insert(self.current_tab + 1, tab);
        self.set_current_tab(self.current_tab + 1);
    }

    pub fn tab_close(&mut self) {
        if self.tabs.len() == 1 {
            self.emit_message(String::from("Cannot close last tab"));
            return;
        }
        for &id in &self.tabs[self.current_tab].open_windows {
            self.windows[id].is_open = false;
        }
        self.tabs.remove(self.current_tab);
        if self.current_tab == self.tabs.len() {
            self.set_current_tab(self.current_tab - 1);
        }
    }

    pub fn tab_next(&mut self) {
        self.set_current_tab(util::rotate_forward(0, self.tabs.len(), self.current_tab));
    }

    pub fn tab_previous(&mut self) {
        self.set_current_tab(util::rotate_backward(0, self.tabs.len(), self.current_tab));
    }

    pub fn move_cursor(&mut self, direction: util::Direction) {
        let window = &mut self.windows[self.tabs[self.current_tab].window_focus];
        window.cursor = window.cursor.move_toward(direction);
        window.keep_cursor_within_bounds();
    }

    pub fn rotate_focus_forward(&mut self) {
        let index = crate::indexvec::VecIndex::get(self.window_focus());
        let index = util::rotate_forward(0, self.windows.len(), index);
        self.set_window_focus(crate::indexvec::VecIndex::new(index));
    }

    pub fn rotate_focus_backward(&mut self) {
        let index = crate::indexvec::VecIndex::get(self.window_focus());
        let index = util::rotate_backward(0, self.windows.len(), index);
        self.set_window_focus(crate::indexvec::VecIndex::new(index));
    }

    fn run_cursor_focus_beam(&self, beam: impl Iterator<Item = Position>) -> Option<WindowID> {
        let window_focus = self.window_focus();
        beam.flat_map(|position| {
            self.tabs[self.current_tab]
                .open_windows
                .iter()
                .copied()
                .filter(move |&id| id != window_focus && self.windows[id].contains(position))
        })
        .next()
    }

    fn send_cursor_focus_beam(&self, direction: util::Direction) -> Option<WindowID> {
        let window = &self.windows[self.tabs[self.current_tab].window_focus];
        let cursor = window.position.offset(window.cursor);

        match direction {
            util::Direction::Up => {
                let range = 0..window.position.y;
                self.run_cursor_focus_beam(range.rev().map(|y| Position { x: cursor.x, y }))
            }
            util::Direction::Down => {
                let range = window.position.y..self.size.height;
                self.run_cursor_focus_beam(range.map(|y| Position { x: cursor.x, y }))
            }
            util::Direction::Left => {
                let range = 0..window.position.x;
                self.run_cursor_focus_beam(range.rev().map(|x| Position { x, y: cursor.y }))
            }
            util::Direction::Right => {
                let range = window.position.x + window.size.width..self.size.width;
                self.run_cursor_focus_beam(range.map(|x| Position { x, y: cursor.y }))
            }
        }
    }

    pub fn move_focus(&mut self, direction: util::Direction) {
        self.send_cursor_focus_beam(direction).inspect(|&id| self.set_window_focus(id));
    }

    pub fn vertical_split_window(&mut self) {
        let left = &mut self.windows[self.tabs[self.current_tab].window_focus];
        if left.size.width < 6 {
            self.emit_message(String::from("The window is too small for a vertical split"));
            return;
        }
        let remainder = left.size.width % 2;
        left.size.width /= 2;
        left.keep_cursor_within_bounds();
        let mut right: Window = *left;
        right.size.width += remainder;
        right.position.x += left.size.width;
        (left.redraw, right.redraw) = (true, true);
        self.tabs[self.current_tab].open_windows.push(self.windows.push(right));
    }

    pub fn horizontal_split_window(&mut self) {
        let above = &mut self.windows[self.tabs[self.current_tab].window_focus];
        if above.size.height < 6 {
            self.emit_message(String::from("The window is too small for a horizontal split"));
            return;
        }
        let remainder = above.size.height % 2;
        above.size.height /= 2;
        above.keep_cursor_within_bounds();
        let mut below: Window = *above;
        below.size.height += remainder;
        below.position.y += above.size.height;
        (above.redraw, below.redraw) = (true, true);
        self.tabs[self.current_tab].open_windows.push(self.windows.push(below));
    }
}
