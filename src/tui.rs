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
use tui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};
use tui::{Frame, Terminal};

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

        let input = Paragraph::new(ui_data.preedit.as_ref())
            .style(Style::default().fg(Color::Yellow))
            .block(Block::default().borders(Borders::ALL).title("Preedit"));
        f.render_widget(input, chunks[0]);
        // f.set_cursor(chunks[0].x + app.input.width() as u16 + 1, chunks[0].y + 1);

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)].as_ref())
            .split(chunks[1]);

        let message = Paragraph::new(ui_data.output.as_ref())
            .block(Block::default().borders(Borders::ALL).title("Message"));
        f.render_widget(message, chunks[0]);

        let items = ui_data
            .candidates
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let mut item = ListItem::new(format!("{}. {}{}", i + 1, c.text, c.comment));
                if c.highlighted {
                    item = item.style(Style::default().fg(Color::Black).bg(Color::White));
                }
                item
            })
            .collect::<Vec<_>>();
        let list =
            List::new(items).block(Block::default().borders(Borders::ALL).title("Candidates"));
        f.render_stateful_widget(list, chunks[1], &mut ListState::default());
    }
}
