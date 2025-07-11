use std::{
    io,
    time::{Duration, Instant},
    vec,
};

use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{Event, KeyCode},
    layout::{Constraint, Layout},
    style::{Color, Style, Stylize},
    symbols::{border, line},
    text::{Line, Span},
    widgets::{Block, Gauge, Paragraph, Widget},
};
fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let mut app = App {
        exit: false,
        new_message_text: vec!["".to_string()],
    };
    let app_result = app.run(&mut terminal);
    //back into normal mode
    ratatui::restore();
    app_result
}

pub struct App {
    exit: bool,
    new_message_text: Vec<String>,
}
impl App {
    fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        let mut last_tab_time: Option<Instant> = None;
        while !self.exit {
            if crossterm::event::poll(Duration::from_millis(100)).unwrap() {
                if let Event::Key(key_event) = ratatui::crossterm::event::read().unwrap() {
                    match key_event.code {
                        (KeyCode::Tab) => {
                            last_tab_time = Some(Instant::now());
                        }
                        (KeyCode::Enter) => {
                            let dlu = self.new_message_text.len();
                            if let Some(time) = last_tab_time {
                                if time.elapsed() < Duration::from_millis(500) {
                                    self.new_message_text.push("amogus".to_string());
                                } else {
                                    self.new_message_text[dlu - 1].push('k');
                                }
                            } else {
                                self.new_message_text[dlu - 1].push('k');
                            }
                        }
                        (KeyCode::Esc) => {
                            self.exit = true;
                        }
                        (KeyCode::Backspace) => {
                            let dlu = self.new_message_text.len();

                            self.new_message_text[dlu - 1].pop();
                        }
                        (KeyCode::Char(c)) => {
                            let dlu = self.new_message_text.len();

                            self.new_message_text[dlu - 1].push(c);
                        }
                        _ => {}
                    }
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
        let xd: Vec<Line> = self
            .new_message_text
            .iter()
            .map(|line| Line::from(Span::raw(" ".to_string().clone() + line)))
            .collect();
        Paragraph::new(xd).block(block).render(typing_area, buf);

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
