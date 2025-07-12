use std::{
    io,
    time::{Duration, Instant},
    vec,
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
        length: 0,
        cursor_position: 0,
    };
    let app_result = app.run(&mut terminal);
    //back into normal mode
    ratatui::restore();
    app_result
}

pub struct App {
    exit: bool,
    new_message_text: Vec<String>,
    length: u16,
    cursor_position: u16,
}
impl App {
    fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        let mut last_tab_time: Option<Instant> = None;
        // Initial draw to show starting state
        //terminal.draw(|frame| self.draw(frame))?;
        while !self.exit {
            // Block until event occurs (no timeout)
            if poll(Duration::from_millis(3))? {
                if let Event::Key(key_event) = ratatui::crossterm::event::read()? {
                    match key_event.code {
                        KeyCode::Tab => {
                            last_tab_time = Some(Instant::now());
                        }
                        KeyCode::Enter => {
                            let dlu = self.new_message_text.len();
                            if let Some(time) = last_tab_time {
                                if time.elapsed() < Duration::from_millis(500) {
                                    if (self.length == self.cursor_position) {
                                        self.cursor_position += 1
                                    }

                                    self.length += 1;

                                    self.new_message_text.push("".to_string());
                                } else {
                                    //todo sent
                                }
                            } else {
                                //todo sent
                            }
                        }
                        KeyCode::Esc => {
                            self.exit = true;
                        }
                        KeyCode::Backspace => {
                            let dlu = self.new_message_text.len();
                            let dlu_linii = self.new_message_text[dlu - 1].len();
                            if dlu_linii > 0 {
                                if (self.length == self.cursor_position) {
                                    self.cursor_position -= 1;
                                }

                                self.length -= 1;

                                self.new_message_text[dlu - 1].pop();
                            } else if dlu > 1 {
                                if (self.length == self.cursor_position) {
                                    self.cursor_position -= 1;
                                }

                                self.length -= 1;

                                self.new_message_text.pop();
                            }
                        }
                        KeyCode::Char(c) => {
                            let dlu = self.new_message_text.len();
                            if (self.length == self.cursor_position) {
                                self.cursor_position += 1;
                            }

                            self.length += 1;

                            self.new_message_text[dlu - 1].push(c);
                            //println!("{c}");
                        }
                        _ => {}
                    }
                    // Single draw call AFTER state update
                }
            }
            terminal.draw(|frame| self.draw(frame))?;
        }
        Ok(())
    }
    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }
}

impl Widget for &App {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        //before i-th letter
        fn place_in_position(input: Vec<String>, position: u16, znak: char) -> Vec<String> {
            let mut output: Vec<String> = input.clone();
            let mut curr_position: u16 = 0;
            for i in 0..input.len() {
                let line = &input[i];
                for j in 0..line.len() {
                    if curr_position == position {
                        output[i].insert(j, znak);
                        return output;
                    }
                    curr_position += 1;
                }
                if curr_position == position {
                    output[i].push(znak);
                    return output;
                }
                curr_position += 1;
            }
            output
        }

        let dlu = self.new_message_text.len();

        let vertical_layout =
            Layout::vertical([Constraint::Percentage(80), Constraint::Min(dlu as u16 + 2)]);
        let [_title_area, typing_area] = vertical_layout.areas(area);
        let instructions = Line::from(vec![
            " change color ".into(),
            "<C> ".blue(),
            "Quit ".into(),
            "<ESC> ".blue(),
        ])
        .centered();
        let block = Block::bordered()
            .title(" typing ")
            .title_bottom(instructions)
            .border_set(border::ROUNDED);

        let mut wiadomosc: Vec<String> = self.new_message_text.clone();
        //wiadomosc[dlu - 1].push('|');
        wiadomosc = place_in_position(wiadomosc, self.cursor_position, '|');
        wiadomosc = wiadomosc
            .iter()
            .map(|line| " ".to_string().clone() + line)
            .collect();

        let wiadomosc_as_spans: Vec<Line> = wiadomosc
            .iter()
            .map(|line| Line::from(Span::raw(line)))
            .collect();
        Paragraph::new(wiadomosc_as_spans)
            .block(block)
            .render(typing_area, buf);

        /*
        let progress_bar = Gauge::default()
            .block(block)
            .style(Style::default().fg(self.progress_bar_color))
            .ratio(0.5);
        progress_bar.render(
            Rect {
                x: typing_area.left(),
                y: typing_area.top(),
                width: typing_area.width,
                height: 3,
            },
            buf,
        );
        */
    }
}
