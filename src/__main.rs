use std::io;

use ratatui::{
    Terminal,
    crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    style::Stylize,
    widgets::Paragraph,
};

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    terminal.clear()?;
    let app_result = run(&mut terminal);
    ratatui::restore();
    app_result
}

fn run(terminal: &mut Terminal<impl ratatui::backend::Backend>) -> io::Result<()> {
    let mut text = String::new();

    loop {
        terminal.draw(|frame| {
            let greeting = Paragraph::new(text.clone()).white().on_blue();
            frame.render_widget(greeting, frame.area());
        })?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                // Obsługa kombinacji Ctrl + klawisz
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    match key.code {
                        KeyCode::Char('c') => return Ok(()),
                        KeyCode::Char('d') => return Ok(()),
                        KeyCode::Enter => text.push_str("Ctrl+Z"),
                        _ => text = format!("Ctrl + {:?}", key.code),
                    }
                    continue;
                }

                // Obsługa pozostałych klawiszy

                match key.code {
                    KeyCode::Esc => return Ok(()),
                    KeyCode::Enter => {
                        if key.modifiers.contains(KeyModifiers::SHIFT) {
                            text.push_str("Shift+Enter\n");
                        } else {
                            text.push_str("Enter\n");
                        }
                    }
                    KeyCode::Char(c) => text.push(c),
                    _ => text = format!("{:?}", key.code),
                }
            }
        }
    }
}
