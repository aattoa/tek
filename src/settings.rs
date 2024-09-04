#![allow(clippy::derivable_impls)]

#[derive(Clone, Copy, Debug)]
pub struct WindowBorders {
    pub top_left: char,
    pub top_right: char,
    pub bottom_left: char,
    pub bottom_right: char,
    pub top_bar: char,
    pub bottom_bar: char,
    pub left_bar: char,
    pub right_bar: char,
}

#[derive(Clone, Copy, Debug)]
pub struct WindowSettings {
    pub borders: WindowBorders,
    pub number: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct EditorSettings {
    pub showmode: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct BufferSettings {
    pub modifiable: bool,
}

impl Default for EditorSettings {
    fn default() -> Self {
        EditorSettings { showmode: true }
    }
}

impl Default for BufferSettings {
    fn default() -> Self {
        BufferSettings { modifiable: false }
    }
}

impl Default for WindowSettings {
    fn default() -> Self {
        WindowSettings { borders: WindowBorders::unicode(), number: true }
    }
}

impl WindowBorders {
    #[rustfmt::skip]
    const fn ascii() -> WindowBorders {
        WindowBorders {
            top_left:     '+',
            top_right:    '+',
            bottom_left:  '+',
            bottom_right: '+',
            top_bar:      '-',
            bottom_bar:   '-',
            left_bar:     '|',
            right_bar:    '|',
        }
    }
    #[rustfmt::skip]
    const fn unicode() -> WindowBorders {
        WindowBorders {
            top_left:     '┌',
            top_right:    '┐',
            bottom_left:  '└',
            bottom_right: '┘',
            top_bar:      '─',
            bottom_bar:   '─',
            left_bar:     '│',
            right_bar:    '│',
        }
    }
}
