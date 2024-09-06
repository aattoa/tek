use crate::editor;
use crate::terminal::{self, Position};
use crate::util::Direction;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::{cursor, style};
use std::io;

pub struct UI {
    editor: editor::Editor,
    command_line: String,
    quit: bool,
}

fn draw_status_line(ui: &UI) -> io::Result<()> {
    terminal::set_cursor(Position { x: 0, y: ui.editor.size.height })?;
    terminal::queue(style::SetBackgroundColor(style::Color::DarkGrey))?;
    terminal::clear_line()?;

    if ui.editor.mode == editor::Mode::CommandLine {
        print!(":{} ", ui.command_line);
    }
    else {
        if ui.editor.settings.showmode {
            print!("-- {:?} -- ", ui.editor.mode);
        }
        if let Some(string) = &ui.editor.status {
            print!("{string} ");
        }
    }
    print!("tab:{}/{} ", ui.editor.current_tab + 1, ui.editor.tabs.len());
    let cursor = ui.editor.windows[ui.editor.window_focus()].cursor;
    print!("cursor:{},{} ", cursor.x + 1, cursor.y + 1);

    terminal::queue(style::SetBackgroundColor(style::Color::Reset))
}

fn line_view(line: &str, view: editor::View) -> &str {
    let from = view.offset as usize;
    let to = from + view.size.width as usize;
    line.get(from..to).unwrap_or(line)
}

fn draw_view(ui: &UI, view: editor::View, position: Position) -> io::Result<()> {
    let text: String = ui.editor.buffers[view.buffer].text.gather();
    let lines: Vec<&str> = text.lines().collect();
    let number_width = lines.len().to_string().len();
    for (index, &line) in lines.iter().enumerate() {
        terminal::set_cursor(position.offset_y(index as u16))?;
        terminal::queue(style::SetForegroundColor(style::Color::DarkGrey))?;
        terminal::queue(style::SetAttribute(style::Attribute::Bold))?;
        print!("{:number_width$}", index + 1);
        terminal::queue(style::SetAttribute(style::Attribute::Reset))?;
        terminal::queue(style::SetForegroundColor(style::Color::Reset))?;
        print!(" {}", line_view(line, view));
    }
    Ok(())
}

fn draw_horizontal_bar(left: char, right: char, middle: char, width: u16) -> io::Result<()> {
    print!("{left}");
    for _ in 0..width - 2 {
        print!("{middle}");
    }
    print!("{right}");
    Ok(())
}

fn draw_window(ui: &UI, window: &editor::Window, focus: bool) -> io::Result<()> {
    if !focus {
        terminal::queue(style::SetForegroundColor(style::Color::DarkGrey))?;
    }
    terminal::set_cursor(window.position)?;
    draw_horizontal_bar(
        window.settings.borders.top_left,
        window.settings.borders.top_right,
        window.settings.borders.top_bar,
        window.size.width,
    )?;
    terminal::set_cursor(window.position.offset_y(window.size.height - 1))?;
    draw_horizontal_bar(
        window.settings.borders.bottom_left,
        window.settings.borders.bottom_right,
        window.settings.borders.bottom_bar,
        window.size.width,
    )?;
    for y in 1..window.size.height - 1 {
        terminal::set_cursor(window.position.offset_y(y))?;
        draw_horizontal_bar(
            window.settings.borders.left_bar,
            window.settings.borders.right_bar,
            ' ',
            window.size.width,
        )?;
    }
    if !focus {
        terminal::queue(style::SetForegroundColor(style::Color::Reset))?;
    }
    if let Some(view) = window.view {
        draw_view(ui, view, window.position.offset_x(1).offset_y(1))?;
    }
    Ok(())
}

fn draw_windows(ui: &mut UI) -> io::Result<()> {
    let window_focus = ui.editor.window_focus();
    for &id in &ui.editor.tabs[ui.editor.current_tab].open_windows {
        if ui.editor.windows[id].redraw {
            draw_window(ui, &ui.editor.windows[id], window_focus == id)?;
            ui.editor.windows[id].redraw = false;
        }
    }
    draw_status_line(ui)
}

fn compute_current_cursor(ui: &UI) -> Position {
    if ui.editor.mode == editor::Mode::CommandLine {
        Position {
            x: ui
                .command_line
                .len()
                .try_into()
                .unwrap_or(ui.editor.size.width.saturating_sub(1))
                .saturating_add(1),
            y: ui.editor.size.height.saturating_sub(1),
        }
    }
    else {
        let window = &ui.editor.windows[ui.editor.tabs[ui.editor.current_tab].window_focus];
        window.position.offset(window.cursor)
    }
}

fn draw(ui: &mut UI) -> io::Result<()> {
    terminal::queue(cursor::Hide)?;
    draw_windows(ui)?;
    terminal::set_cursor(compute_current_cursor(ui))?;
    terminal::queue(cursor::Show)?;
    terminal::flush()
}

fn execute_command_line(ui: &mut UI) -> io::Result<()> {
    let mut pieces = ui.command_line.split_whitespace();
    if let Some(command) = pieces.next() {
        match command {
            "e" | "edit" => {
                if let Some(argument) = pieces.next() {
                    ui.editor.edit(argument.into())?;
                }
            }
            "q" | "quit" => ui.quit = true,
            "sp" | "split" => ui.editor.horizontal_split_window(),
            "vsp" | "vsplit" => ui.editor.vertical_split_window(),
            "redraw" => ui.editor.force_redraw(),
            "tabopen" => ui.editor.tab_open(),
            "tabclose" => ui.editor.tab_close(),
            "tabnext" => ui.editor.tab_next(),
            "tabprev" | "tabprevious" => ui.editor.tab_previous(),
            _ => ui.editor.status = Some(format!("Unrecognized command: {command}")),
        }
    }
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
            KeyCode::Char('q') if key.modifiers == KeyModifiers::CONTROL => {
                ui.editor.tab_close();
            }
            KeyCode::Char('t') if key.modifiers == KeyModifiers::CONTROL => {
                ui.editor.tab_open();
            }
            KeyCode::Char(character) => match character {
                'h' => ui.editor.move_cursor(Direction::Left),
                'j' => ui.editor.move_cursor(Direction::Down),
                'k' => ui.editor.move_cursor(Direction::Up),
                'l' => ui.editor.move_cursor(Direction::Right),
                'H' => ui.editor.tab_previous(),
                'L' => ui.editor.tab_next(),
                'i' => ui.editor.mode = editor::Mode::Insert,
                ':' => ui.editor.mode = editor::Mode::CommandLine,
                _ => {}
            },
            _ => {}
        },
        editor::Mode::Window => match key.code {
            KeyCode::Esc => ui.editor.mode = editor::Mode::Normal,
            KeyCode::Char(character) => match character {
                'h' => ui.editor.move_focus(Direction::Left),
                'j' => ui.editor.move_focus(Direction::Down),
                'k' => ui.editor.move_focus(Direction::Up),
                'l' => ui.editor.move_focus(Direction::Right),
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
            KeyCode::Char(character) => ui.editor.status = Some(format!("got '{character}'")),
            _ => {}
        },
        editor::Mode::CommandLine => match key.code {
            KeyCode::Esc => ui.editor.mode = editor::Mode::Normal,
            KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => {
                if ui.command_line.is_empty() {
                    ui.editor.mode = editor::Mode::Normal;
                }
                else {
                    ui.command_line.clear();
                }
            }
            KeyCode::Char(character) => {
                ui.command_line.push(character);
            }
            KeyCode::Backspace => {
                ui.command_line.pop();
            }
            KeyCode::Enter => {
                execute_command_line(ui)?;
                ui.command_line.clear();
                ui.editor.mode = editor::Mode::Normal;
            }
            _ => {}
        },
    }

    Ok(())
}

fn handle_event(ui: &mut UI, event: Event) -> io::Result<()> {
    match event {
        Event::Key(event) => {
            handle_key(ui, event)?;
        }
        Event::Resize(width, height) => {
            ui.editor.size = terminal::Size { width, height };
        }
        _ => {}
    }
    Ok(())
}

impl UI {
    pub fn new(size: terminal::Size) -> UI {
        UI {
            editor: editor::Editor::new(size),
            command_line: String::new(),
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
}
