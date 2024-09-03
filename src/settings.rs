#![allow(clippy::derivable_impls)]

#[derive(Clone, Copy, Debug)]
pub struct WindowBorders {
    top_left: char,
    top_right: char,
    bottom_left: char,
    bottom_right: char,
    top_bar: char,
    bottom_bar: char,
    left_bar: char,
    right_bar: char,
}

#[derive(Clone, Copy, Debug)]
pub struct Editor {
    pub showmode: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct Buffer {
    pub modifiable: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct Window {
    pub borders: WindowBorders,
    pub number: bool,
}

impl Default for Editor {
    fn default() -> Self {
        Editor { showmode: true }
    }
}

impl Default for Buffer {
    fn default() -> Self {
        Buffer { modifiable: false }
    }
}

impl Default for Window {
    fn default() -> Self {
        Window { borders: WindowBorders::unicode(), number: true }
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
