use std::{
    io,
    time::{Duration, Instant},
};

use ratatui::{
    DefaultTerminal, Frame,
    crossterm::{
        event::{Event, KeyCode, poll},
        terminal::enable_raw_mode,
    },
    layout::{Constraint, Layout},
    style::Stylize,
    symbols::border,
    text::{Line, Span},
    widgets::{Block, Paragraph, Widget},
};

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    enable_raw_mode()?;

    let mut app = App {
        exit: false,
        new_message_text: vec!["".to_string()],
        cursor_position: 0,
    };
    let app_result = app.run(&mut terminal);
    ratatui::restore();
    app_result
}

pub struct App {
    exit: bool,
    new_message_text: Vec<String>,
    cursor_position: usize,
}

impl App {
    fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        let mut last_tab_time: Option<Instant> = None;
        while !self.exit {
            if poll(Duration::from_millis(3))? {
                if let Event::Key(key_event) = ratatui::crossterm::event::read()? {
                    match key_event.code {
                        KeyCode::Tab => {
                            last_tab_time = Some(Instant::now());
                        }
                        KeyCode::Enter => {
                            if let Some(time) = last_tab_time {
                                if time.elapsed() < Duration::from_millis(500) {
                                    let (line_idx, char_idx) = self.get_cursor_line_char_index();

                                    // Compute byte index without holding immutable borrow
                                    let byte_idx = {
                                        let line = &self.new_message_text[line_idx];
                                        line.char_indices()
                                            .nth(char_idx)
                                            .map(|(i, _)| i)
                                            .unwrap_or_else(|| line.len())
                                    };

                                    // Now work directly with mutable access
                                    let right_part =
                                        self.new_message_text[line_idx].split_off(byte_idx);
                                    self.new_message_text.insert(line_idx + 1, right_part);
                                    // Move cursor to beginning of new line
                                    self.cursor_position += 1;
                                } else {
                                    // todo: send message
                                }
                            } else {
                                // todo: send message
                            }
                        }
                        KeyCode::Esc => {
                            self.exit = true;
                        }
                        KeyCode::Backspace => {
                            if self.cursor_position > 0 {
                                let (line_idx, char_idx) = self.get_cursor_line_char_index();

                                if char_idx > 0 {
                                    let line = &mut self.new_message_text[line_idx];

                                    // Find byte index of character to remove
                                    let (byte_idx, c) = line
                                        .char_indices()
                                        .nth(char_idx - 1)
                                        .expect("Character should exist");

                                    // Calculate how many bytes to remove
                                    let char_len = c.len_utf8();

                                    // Remove the character
                                    line.drain(byte_idx..byte_idx + char_len);
                                    self.cursor_position -= 1;
                                } else if line_idx > 0 {
                                    // Merge with previous line
                                    let current_line = self.new_message_text.remove(line_idx);
                                    let prev_line = &mut self.new_message_text[line_idx - 1];
                                    prev_line.push_str(&current_line);
                                    self.cursor_position -= 1;
                                }
                            }
                        }
                        // Delete key support
                        KeyCode::Delete => {
                            let total_chars = self.get_total_chars();
                            if self.cursor_position < total_chars {
                                let (line_idx, char_idx) = self.get_cursor_line_char_index();
                                let line_char_count =
                                    self.new_message_text[line_idx].chars().count();

                                if char_idx < line_char_count {
                                    // Delete within line
                                    let line = &mut self.new_message_text[line_idx];
                                    let (byte_idx, c) = line
                                        .char_indices()
                                        .nth(char_idx)
                                        .expect("Character should exist");
                                    let char_len = c.len_utf8();
                                    line.drain(byte_idx..byte_idx + char_len);
                                } else if line_idx < self.new_message_text.len() - 1 {
                                    // At end of line - merge with next line
                                    let next_line = self.new_message_text.remove(line_idx + 1);
                                    let line = &mut self.new_message_text[line_idx];
                                    line.push_str(&next_line);
                                }
                            }
                        }
                        KeyCode::Char(c) => {
                            let (line_idx, char_idx) = self.get_cursor_line_char_index();
                            let line = &mut self.new_message_text[line_idx];

                            // Find byte index for insertion
                            let byte_idx = line
                                .char_indices()
                                .nth(char_idx)
                                .map(|(i, _)| i)
                                .unwrap_or_else(|| line.len());

                            // Insert character at byte position
                            line.insert(byte_idx, c);
                            self.cursor_position += 1;
                        }
                        // Arrow key handling
                        KeyCode::Left => {
                            if self.cursor_position > 0 {
                                self.cursor_position -= 1;
                            }
                        }
                        KeyCode::Right => {
                            // Calculate total characters in text including newlines
                            let total_chars = self.get_total_chars();

                            if self.cursor_position < total_chars {
                                self.cursor_position += 1;
                            }
                        }
                        KeyCode::Up => {
                            let (current_line, current_col) = self.get_cursor_line_char_index();

                            if current_line > 0 {
                                let prev_line = &self.new_message_text[current_line - 1];
                                let prev_line_chars = prev_line.chars().count();

                                // Find position in previous line
                                let new_col = current_col.min(prev_line_chars);

                                // Calculate new cursor position
                                let mut new_position = 0;
                                for i in 0..(current_line - 1) {
                                    new_position += self.new_message_text[i].chars().count();
                                }
                                new_position += new_col;

                                // Add newline positions
                                new_position += current_line - 1;

                                self.cursor_position = new_position;
                            }
                        }
                        KeyCode::Down => {
                            let (current_line, current_col) = self.get_cursor_line_char_index();

                            if current_line < self.new_message_text.len() - 1 {
                                let next_line = &self.new_message_text[current_line + 1];
                                let next_line_chars = next_line.chars().count();

                                // Find position in next line
                                let new_col = current_col.min(next_line_chars);

                                // Calculate new cursor position
                                let mut new_position = 0;
                                for i in 0..=current_line {
                                    new_position += self.new_message_text[i].chars().count();
                                }
                                new_position += new_col;

                                // Add newline positions
                                new_position += current_line + 1;

                                self.cursor_position = new_position;
                            }
                        }
                        // Home key - move to beginning of line
                        KeyCode::Home => {
                            let (line_idx, _) = self.get_cursor_line_char_index();
                            self.cursor_position = self.get_line_start(line_idx);
                        }
                        // End key - move to end of line
                        KeyCode::End => {
                            let (line_idx, _) = self.get_cursor_line_char_index();
                            self.cursor_position = self.get_line_end(line_idx);
                        }
                        _ => {}
                    }
                }
            }
            terminal.draw(|frame| self.draw(frame))?;
        }
        Ok(())
    }

    // Helper to get total characters including newlines
    fn get_total_chars(&self) -> usize {
        let mut total = 0;
        for (i, line) in self.new_message_text.iter().enumerate() {
            total += line.chars().count();
            if i < self.new_message_text.len() - 1 {
                total += 1; // Account for newline
            }
        }
        total
    }

    // Get the start position of a line (global index) including newlines
    fn get_line_start(&self, line_idx: usize) -> usize {
        let mut position = 0;
        for (_i, line) in self.new_message_text.iter().enumerate().take(line_idx) {
            position += line.chars().count();
            position += 1; // Newline
        }
        position
    }

    // Get the end position of a line (global index) including newlines
    fn get_line_end(&self, line_idx: usize) -> usize {
        let mut position = self.get_line_start(line_idx);
        position += self.new_message_text[line_idx].chars().count();
        position
    }

    // Get line and character index within the line, accounting for newlines
    fn get_cursor_line_char_index(&self) -> (usize, usize) {
        let mut chars_remaining = self.cursor_position;
        for (line_idx, line) in self.new_message_text.iter().enumerate() {
            let line_char_count = line.chars().count();
            let line_total = line_char_count
                + if line_idx < self.new_message_text.len() - 1 {
                    1
                } else {
                    0
                };

            if chars_remaining < line_total {
                return (line_idx, chars_remaining.min(line_char_count));
            }
            chars_remaining -= line_total;
        }
        // Default to end of last line
        let last_idx = self.new_message_text.len() - 1;
        (last_idx, self.new_message_text[last_idx].chars().count())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }
}

impl Widget for &App {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        // Helper to insert cursor at character position
        fn insert_cursor(text: &[String], cursor_pos: usize) -> Vec<String> {
            let mut result = text.to_vec();
            let mut remaining_pos = cursor_pos;

            for (line_idx, line) in text.iter().enumerate() {
                let line_char_count = line.chars().count();
                let line_total = line_char_count + if line_idx < text.len() - 1 { 1 } else { 0 };

                if remaining_pos < line_total {
                    // Handle newline position
                    if remaining_pos == line_char_count && line_idx < text.len() - 1 {
                        // Special handling for newline position
                        result[line_idx] = line.clone() + "|";
                        return result;
                    }

                    // Find byte index for insertion
                    let char_pos = remaining_pos.min(line_char_count);
                    let byte_idx = line
                        .char_indices()
                        .nth(char_pos)
                        .map(|(i, _)| i)
                        .unwrap_or_else(|| line.len());

                    // Create new line with cursor inserted
                    let mut new_line = line.clone();
                    new_line.insert(byte_idx, '|');
                    result[line_idx] = new_line;
                    return result;
                }

                remaining_pos -= line_total;
            }

            // If we reach here, add cursor to last line
            if let Some(last) = result.last_mut() {
                *last = last.clone() + "|";
            }
            result
        }

        let vertical_layout = Layout::vertical([Constraint::Percentage(80), Constraint::Min(3)]);
        let [_title_area, typing_area] = vertical_layout.areas(area);
        let instructions = Line::from(vec![
            "Delete ".into(),
            "<Del> ".blue(),
            "Quit ".into(),
            "<ESC> ".blue(),
        ])
        .centered();
        let block = Block::bordered()
            .title(" typing ")
            .title_bottom(instructions)
            .border_set(border::ROUNDED);

        // Insert cursor at correct position
        let mut wiadomosc = insert_cursor(&self.new_message_text, self.cursor_position);

        // Add padding to each line
        wiadomosc = wiadomosc
            .iter()
            .map(|line| " ".to_string() + line)
            .collect();

        // Convert to display format
        let wiadomosc_as_spans: Vec<Line> = wiadomosc
            .iter()
            .map(|line| Line::from(Span::raw(line)))
            .collect();

        Paragraph::new(wiadomosc_as_spans)
            .block(block)
            .render(typing_area, buf);
    }
}
