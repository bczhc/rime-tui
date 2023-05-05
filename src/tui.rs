use std::io;
use std::io::{stdout, Stdout};
use std::thread::spawn;

use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{event, execute};
use tui::backend::{Backend, CrosstermBackend};
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Style};
use tui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};
use tui::{Frame, Terminal};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

pub struct TuiApp<B>
where
    B: Backend,
{
    pub ui_data: UiData,
    terminal: Terminal<B>,
}

#[derive(Debug, Default)]
pub struct Candidate {
    pub text: String,
    pub comment: String,
    pub highlighted: bool,
}

#[derive(Debug, Default)]
pub struct UiData {
    pub preedit: String,
    pub candidates: Vec<Candidate>,
    pub output: String,
    pub log: Vec<String>,
    pub select_labels: Option<Vec<String>>,
}

impl TuiApp<CrosstermBackend<Stdout>> {
    pub fn new() -> io::Result<Self> {
        let stdout = stdout();
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        Ok(Self {
            ui_data: Default::default(),
            terminal,
        })
    }

    pub fn start(&self) -> io::Result<()> {
        enable_raw_mode()?;
        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        // consume terminal key input events
        // Because we are getting keyboard events from X11 APIs, but not via this,
        // so if don't do this, when the app is terminated, the screen will leave
        // all the text the user just inputted.
        // This method is a bit tricky but just works. And I haven't found a better way (I can't
        // get `event:poll` work).
        spawn(|| loop {
            event::read().unwrap();
        });
        Ok(())
    }

    pub fn stop(&mut self) -> io::Result<()> {
        let terminal = &mut self.terminal;
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;
        Ok(())
    }

    pub fn redraw(&mut self) -> io::Result<()> {
        self.terminal.draw(|f| Self::ui(&self.ui_data, f))?;
        Ok(())
    }

    fn ui<B: Backend>(ui_data: &UiData, f: &mut Frame<B>) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([Constraint::Length(3), Constraint::Min(1)].as_ref())
            .split(f.size());

        let preedit_chunk = chunks[0];

        let input = Paragraph::new(ui_data.preedit.as_ref())
            .style(Style::default().fg(Color::Yellow))
            .block(Block::default().borders(Borders::ALL).title("Preedit"));
        f.render_widget(input, preedit_chunk);

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)].as_ref())
            .split(chunks[1]);

        let candidates_chunk = chunks[1];

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)].as_ref())
            .split(chunks[0]);

        let message_chunk = chunks[0];
        let log_chunk = chunks[1];

        let message_width = message_chunk.width - 2 /* border size takes 2 */;
        let message_height = message_chunk.height - 2;
        let wrapped_message = wrap_text(&ui_data.output, message_width);
        let message = Paragraph::new(wrapped_message.1)
            .block(Block::default().borders(Borders::ALL).title("Output"))
            .scroll((
                if wrapped_message.0 > message_height {
                    wrapped_message.0 - message_height
                } else {
                    0
                },
                0,
            ));
        f.render_widget(message, message_chunk);

        let items = ui_data
            .candidates
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let label = match &ui_data.select_labels {
                    None => {
                        format!("{}.", i + 1)
                    }
                    Some(l) => l[i].clone(),
                };

                let mut item = ListItem::new(format!("{} {}{}", label, c.text, c.comment));
                if c.highlighted {
                    item = item.style(Style::default().fg(Color::Black).bg(Color::White));
                }
                item
            })
            .collect::<Vec<_>>();
        let list =
            List::new(items).block(Block::default().borders(Borders::ALL).title("Candidates"));
        f.render_stateful_widget(list, candidates_chunk, &mut ListState::default());

        let last_line = ui_data.log.last();
        let last_line = last_line.map(|x| x.as_str()).unwrap_or("");
        let log = Paragraph::new(last_line)
            .block(Block::default().borders(Borders::ALL).title("Log"))
            .wrap(Wrap { trim: false });
        f.render_widget(log, log_chunk);
    }
}

fn wrap_text(text: &str, width: u16) -> (u16, String) {
    let mut wrapped_lines = String::new();
    let mut line_count = 0_u16;

    let mut line = String::new();
    for c in text.chars() {
        if c == '\n' {
            wrapped_lines.push_str(&line);
            wrapped_lines.push('\n');
            line.clear();
            line_count += 1;
            continue;
        }
        line.push(c);
        if line.width() > width as usize - c.width().unwrap_or(1) {
            wrapped_lines.push_str(&line);
            wrapped_lines.push('\n');
            line.clear();
            line_count += 1;
        }
    }
    if !line.is_empty() {
        wrapped_lines.push_str(&line);
        line_count += 1;
    }
    (line_count, wrapped_lines)
}
