use std::{
    io,
    sync::{Arc, Mutex, mpsc},
    thread,
    time::{Duration, Instant},
};

use clap::Parser;
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    crossterm::{
        event::{Event, KeyCode, poll, read},
        execute,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    },
    layout::{Constraint, Layout},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
};
use serde_json;
use tungstenite::{Message, connect};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    nick: String,

    #[arg(short, long, default_value = "localhost:9001")]
    backend: String,
}

fn main() -> io::Result<()> {
    let args = Args::parse();
    let ws_url = format!("ws://{}", args.backend);

    let mut stdout = io::stdout();
    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let (ws_tx, ws_rx) = mpsc::channel::<String>();
    let (ui_tx, ui_rx) = mpsc::channel::<String>();
    let messages = Arc::new(Mutex::new(Vec::<String>::new()));
    let connection_status = Arc::new(Mutex::new(format!("Connecting to {}...", ws_url)));

    let status_clone = Arc::clone(&connection_status);
    let ui_tx_clone = ui_tx.clone();
    let url_clone = ws_url.clone();
    thread::spawn(move || match connect(&url_clone) {
        Ok((mut socket, _)) => {
            *status_clone.lock().unwrap() = format!("Connected to {}", url_clone);

            loop {
                if let Ok(msg) = ws_rx.try_recv() {
                    if let Err(e) = socket.send(Message::Text(msg.into())) {
                        *status_clone.lock().unwrap() = format!("Send error: {}", e);
                        break;
                    }
                }

                match socket.read() {
                    Ok(message) => {
                        if let Message::Text(text) = message {
                            ui_tx_clone.send(text.to_string()).unwrap();
                        }
                    }
                    Err(tungstenite::Error::Io(ref err))
                        if err.kind() == io::ErrorKind::WouldBlock =>
                    {
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(e) => {
                        *status_clone.lock().unwrap() = format!("Receive error: {}", e);
                        break;
                    }
                }
            }
        }
        Err(e) => {
            *status_clone.lock().unwrap() = format!("Connection failed: {}", e);
        }
    });

    let mut app = App {
        exit: false,
        new_message_text: vec!["".to_string()],
        cursor_position: 0,
        messages,
        ws_tx,
        input_history: Vec::new(),
        history_index: 0,
        connection_status,
        scroll_offset: 0,
        scroll_to_bottom: true,
        nick: args.nick,
    };

    let app_result = app.run(&mut terminal, ui_rx);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    app_result
}

pub struct App {
    exit: bool,
    new_message_text: Vec<String>,
    cursor_position: usize,
    messages: Arc<Mutex<Vec<String>>>,
    ws_tx: mpsc::Sender<String>,
    input_history: Vec<String>,
    history_index: usize,
    connection_status: Arc<Mutex<String>>,
    scroll_offset: usize,
    scroll_to_bottom: bool,
    nick: String,
}

impl App {
    fn run(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
        ui_rx: mpsc::Receiver<String>,
    ) -> io::Result<()> {
        let mut last_tab_time: Option<Instant> = None;

        loop {
            while let Ok(msg) = ui_rx.try_recv() {
                let mut messages = self.messages.lock().unwrap();
                messages.push(msg);
                self.scroll_to_bottom = true;
            }

            if poll(Duration::from_millis(16))? {
                if let Event::Key(key_event) = read()? {
                    match key_event.code {
                        KeyCode::Tab => {
                            last_tab_time = Some(Instant::now());
                        }
                        KeyCode::Enter => {
                            if let Some(time) = last_tab_time {
                                if time.elapsed() < Duration::from_millis(500) {
                                    let (line_idx, char_idx) = self.get_cursor_line_char_index();
                                    let byte_idx = {
                                        let line = &self.new_message_text[line_idx];
                                        line.char_indices()
                                            .nth(char_idx)
                                            .map(|(i, _)| i)
                                            .unwrap_or_else(|| line.len())
                                    };

                                    let right_part =
                                        self.new_message_text[line_idx].split_off(byte_idx);
                                    self.new_message_text.insert(line_idx + 1, right_part);
                                    self.cursor_position += 1;
                                } else {
                                    self.send_message();
                                }
                            } else {
                                self.send_message();
                            }
                        }
                        KeyCode::Esc => {
                            self.exit = true;
                        }
                        KeyCode::Up => {
                            if self.new_message_text.len() == 1 {
                                if !self.input_history.is_empty() {
                                    if self.history_index == 0 {
                                        self.history_index = self.input_history.len();
                                    } else {
                                        self.history_index -= 1;
                                    }
                                    if self.input_history.len() > self.history_index {
                                        self.new_message_text[0] =
                                            self.input_history[self.history_index].clone();
                                        self.cursor_position = self.new_message_text[0].len();
                                    }
                                }
                            } else {
                                let (current_line, current_col) = self.get_cursor_line_char_index();
                                if current_line > 0 {
                                    let prev_line = &self.new_message_text[current_line - 1];
                                    let prev_line_chars = prev_line.chars().count();
                                    let new_col = current_col.min(prev_line_chars);

                                    let mut new_position = 0;
                                    for i in 0..(current_line - 1) {
                                        new_position += self.new_message_text[i].chars().count();
                                    }
                                    new_position += new_col;
                                    new_position += current_line - 1;

                                    self.cursor_position = new_position;
                                }
                            }
                        }
                        KeyCode::Down => {
                            if self.new_message_text.len() == 1 {
                                if !self.input_history.is_empty() {
                                    self.history_index =
                                        (self.history_index + 1) % self.input_history.len();
                                    self.new_message_text[0] =
                                        self.input_history[self.history_index].clone();
                                    self.cursor_position = self.new_message_text[0].len();
                                }
                            } else {
                                let (current_line, current_col) = self.get_cursor_line_char_index();
                                if current_line < self.new_message_text.len() - 1 {
                                    let next_line = &self.new_message_text[current_line + 1];
                                    let next_line_chars = next_line.chars().count();
                                    let new_col = current_col.min(next_line_chars);

                                    let mut new_position = 0;
                                    for i in 0..=current_line {
                                        new_position += self.new_message_text[i].chars().count();
                                    }
                                    new_position += new_col;
                                    new_position += current_line;

                                    self.cursor_position = new_position;
                                }
                            }
                        }
                        KeyCode::Left => {
                            if self.cursor_position > 0 {
                                self.cursor_position -= 1;
                            }
                        }
                        KeyCode::Right => {
                            let total_chars = self.get_total_chars();
                            if self.cursor_position < total_chars {
                                self.cursor_position += 1;
                            }
                        }
                        KeyCode::Backspace => {
                            if self.cursor_position > 0 {
                                let (line_idx, char_idx) = self.get_cursor_line_char_index();
                                if char_idx > 0 {
                                    let line = &mut self.new_message_text[line_idx];
                                    let (byte_idx, c) = line
                                        .char_indices()
                                        .nth(char_idx - 1)
                                        .expect("Character should exist");
                                    let char_len = c.len_utf8();
                                    line.drain(byte_idx..byte_idx + char_len);
                                    self.cursor_position -= 1;
                                } else if line_idx > 0 {
                                    let current_line = self.new_message_text.remove(line_idx);
                                    let prev_line = &mut self.new_message_text[line_idx - 1];
                                    prev_line.push_str(&current_line);
                                    self.cursor_position -= 1;
                                }
                            }
                        }
                        KeyCode::Delete => {
                            let total_chars = self.get_total_chars();
                            if self.cursor_position < total_chars {
                                let (line_idx, char_idx) = self.get_cursor_line_char_index();
                                let line_char_count =
                                    self.new_message_text[line_idx].chars().count();
                                if char_idx < line_char_count {
                                    let line = &mut self.new_message_text[line_idx];
                                    let (byte_idx, c) = line
                                        .char_indices()
                                        .nth(char_idx)
                                        .expect("Character should exist");
                                    let char_len = c.len_utf8();
                                    line.drain(byte_idx..byte_idx + char_len);
                                } else if line_idx < self.new_message_text.len() - 1 {
                                    let next_line = self.new_message_text.remove(line_idx + 1);
                                    let line = &mut self.new_message_text[line_idx];
                                    line.push_str(&next_line);
                                }
                            }
                        }
                        KeyCode::Char(c) => {
                            let (line_idx, char_idx) = self.get_cursor_line_char_index();
                            let line = &mut self.new_message_text[line_idx];
                            let byte_idx = line
                                .char_indices()
                                .nth(char_idx)
                                .map(|(i, _)| i)
                                .unwrap_or_else(|| line.len());
                            line.insert(byte_idx, c);
                            self.cursor_position += 1;
                        }
                        KeyCode::Home => {
                            let (line_idx, _) = self.get_cursor_line_char_index();
                            self.cursor_position = self.get_line_start(line_idx);
                        }
                        KeyCode::End => {
                            let (line_idx, _) = self.get_cursor_line_char_index();
                            self.cursor_position = self.get_line_end(line_idx);
                        }
                        _ => {}
                    }
                }
            }

            terminal.draw(|frame| self.draw(frame))?;

            if self.exit {
                break;
            }
        }
        Ok(())
    }

    fn send_message(&mut self) {
        let message = self.new_message_text.join("\n");
        if !message.trim().is_empty() {
            self.input_history.push(message.clone());
            self.history_index = self.input_history.len();

            let json_message = serde_json::json!({
                "nick": self.nick,
                "message": message,
            });
            let json_string = json_message.to_string();

            if let Err(e) = self.ws_tx.send(json_string) {
                *self.connection_status.lock().unwrap() = format!("Send error: {}", e);
            }
        }

        self.new_message_text = vec!["".to_string()];
        self.cursor_position = 0;
    }

    fn get_total_chars(&self) -> usize {
        let mut total = 0;
        for (i, line) in self.new_message_text.iter().enumerate() {
            total += line.chars().count();
            if i < self.new_message_text.len() - 1 {
                total += 1;
            }
        }
        total
    }

    fn get_line_start(&self, line_idx: usize) -> usize {
        let mut position = 0;
        for i in 0..line_idx {
            position += self.new_message_text[i].chars().count();
            position += 1;
        }
        position
    }

    fn get_line_end(&self, line_idx: usize) -> usize {
        let mut position = self.get_line_start(line_idx);
        position += self.new_message_text[line_idx].chars().count();
        position
    }

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
        let last_idx = self.new_message_text.len() - 1;
        (last_idx, self.new_message_text[last_idx].chars().count())
    }

    fn draw(&mut self, frame: &mut Frame) {
        let vertical_layout = Layout::vertical([
            Constraint::Length(3),                                        // Title bar
            Constraint::Min(3),                                           // Messages
            Constraint::Length(2),                                        // Status bar
            Constraint::Length(1),                                        // Input title
            Constraint::Length((2 + self.new_message_text.len()) as u16), // Input area
        ]);

        let [
            title_area,
            messages_area,
            status_area,
            input_title_area,
            input_area,
        ] = vertical_layout.areas(frame.area());

        let title = Block::default()
            .title(" 💬 Rust Chat Client ")
            .title_alignment(ratatui::layout::Alignment::Center)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::LightBlue))
            .style(Style::default().bg(Color::DarkGray));
        frame.render_widget(title, title_area);

        let messages = self.messages.lock().unwrap();

        let mut all_lines = Vec::new();
        for msg in messages.iter() {
            let lines: Vec<&str> = msg.split('\n').collect();
            if !lines.is_empty() {
                for line in lines.iter() {
                    all_lines.push(line.to_string());
                }
            }
        }

        if self.scroll_to_bottom {
            self.scroll_offset = all_lines
                .len()
                .saturating_sub(messages_area.height as usize);
            self.scroll_to_bottom = false;
        }

        let max_scroll = all_lines
            .len()
            .saturating_sub(messages_area.height as usize);
        if self.scroll_offset > max_scroll {
            self.scroll_offset = max_scroll;
        }

        let visible_lines: Vec<&str> = all_lines
            .iter()
            .skip(self.scroll_offset)
            .take(messages_area.height as usize)
            .map(|s| s.as_str())
            .collect();

        let messages_text: Text = Text::from(
            visible_lines
                .iter()
                .map(|line| Line::from(Span::raw(*line)))
                .collect::<Vec<_>>(),
        );

        let msg_widget =
            Paragraph::new(messages_text).block(Block::default().borders(Borders::NONE));

        frame.render_widget(msg_widget, messages_area);

        let status = self.connection_status.lock().unwrap();
        let status_widget = Paragraph::new(status.as_str())
            .block(Block::default().borders(Borders::TOP))
            .style(Style::default().fg(Color::Yellow));
        frame.render_widget(status_widget, status_area);

        let input_title =
            Paragraph::new(" Type your message (Enter to send, Tab+Enter for new line):")
                .style(Style::default().fg(Color::LightCyan));
        frame.render_widget(input_title, input_title_area);

        let input_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Gray));

        fn insert_cursor(text: &[String], cursor_pos: usize) -> Vec<String> {
            let mut result = text.to_vec();
            let mut remaining_pos = cursor_pos;

            for (line_idx, line) in text.iter().enumerate() {
                let line_char_count = line.chars().count();
                let line_total = line_char_count + if line_idx < text.len() - 1 { 1 } else { 0 };

                if remaining_pos < line_total {
                    if remaining_pos == line_char_count && line_idx < text.len() - 1 {
                        result[line_idx] = line.clone() + "│";
                        return result;
                    }

                    let char_pos = remaining_pos.min(line_char_count);
                    let byte_idx = line
                        .char_indices()
                        .nth(char_pos)
                        .map(|(i, _)| i)
                        .unwrap_or_else(|| line.len());

                    let mut new_line = line.clone();
                    new_line.insert_str(byte_idx, "│");
                    result[line_idx] = new_line;
                    return result;
                }

                remaining_pos -= line_total;
            }

            if let Some(last) = result.last_mut() {
                *last = last.clone() + "│";
            }
            result
        }

        let mut wiadomosc = insert_cursor(&self.new_message_text, self.cursor_position);

        wiadomosc = wiadomosc
            .iter()
            .map(|line| " ".to_string() + line)
            .collect();

        let wiadomosc_as_spans: Vec<Line> = wiadomosc
            .iter()
            .map(|line| Line::from(Span::raw(line)))
            .collect();

        let input_widget = Paragraph::new(wiadomosc_as_spans)
            .block(input_block)
            .wrap(Wrap { trim: true });

        frame.render_widget(input_widget, input_area);
    }
}
