use crossterm::terminal::enable_raw_mode;
use crossterm::{event::EnableMouseCapture, execute, terminal::EnterAlternateScreen};
use std::io::Stdout;
use std::io::{self, stdout};
use tui::layout::{Alignment, Constraint, Layout};
use tui::style::{Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::{List, ListItem, ListState, Paragraph, Wrap};
use tui::{
    backend::CrosstermBackend,
    widgets::{Block, Borders},
    Terminal,
};

use crate::state::Mode;
use crate::State;

pub struct RenderState {
    pub files_in_pwd: Vec<String>,
    pub selected_in_pwd: Option<usize>,

    pub files_in_parent: Vec<String>,
    pub selected_in_parent: Option<usize>,

    pub command: Option<String>,
}

impl From<State> for RenderState {
    fn from(other: State) -> Self {
        let command = match &other.mode {
            Mode::NormalMode => None,
            Mode::CommandMode(pr) => Some(pr.result().to_string()),
            Mode::ShellCommandMode(pr) => Some(pr.result().to_string()),
        };

        RenderState {
            files_in_pwd: other
                .files_in_pwd()
                .iter()
                .map(|x| x.file_name().unwrap().to_str().unwrap().to_string())
                .collect(),
            selected_in_pwd: Some(other.selected_index_in_pwd()),
            files_in_parent: other
                .files_in_parent()
                .iter()
                .map(|x| x.file_name().unwrap().to_str().unwrap().to_string())
                .collect(),
            selected_in_parent: other.selected_index_in_parent(),
            command,
        }
    }
}

pub fn create_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>, io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

pub fn draw(
    state: &RenderState,
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
) -> Result<(), io::Error> {
    terminal.draw(|f| {
        let vertical_rects = Layout::default()
            .direction(tui::layout::Direction::Vertical)
            .margin(0)
            .constraints([Constraint::Min(2), Constraint::Length(1)].as_ref())
            .split(f.size());

        let main_window_rects = Layout::default()
            .direction(tui::layout::Direction::Horizontal)
            .margin(0)
            .constraints(
                [
                    Constraint::Percentage(30),
                    Constraint::Percentage(40),
                    Constraint::Percentage(30),
                ]
                .as_ref(),
            )
            .split(vertical_rects[0]);

        let current_list_items: Vec<ListItem> = state
            .files_in_pwd
            .iter()
            .map(|x| ListItem::new(x.to_string()))
            .collect();

        let parent_list_items: Vec<ListItem> = state
            .files_in_parent
            .iter()
            .map(|x| ListItem::new(x.to_string()))
            .collect();

        let parent = List::new(parent_list_items)
            .block(Block::default().title("Parent").borders(Borders::ALL))
            .highlight_style(Style::default().add_modifier(Modifier::BOLD));

        let files = List::new(current_list_items)
            .block(Block::default().title("Files").borders(Borders::ALL))
            .highlight_style(Style::default().add_modifier(Modifier::BOLD))
            .highlight_symbol(">>");

        let preview = Block::default().title("Preview").borders(Borders::ALL);

        let mut files_state = ListState::default();
        files_state.select(state.selected_in_pwd);

        let mut parents_state = ListState::default();
        parents_state.select(state.selected_in_parent);

        let command_window_text = vec![Spans::from(vec![Span::raw(
            state.command.clone().unwrap_or("".into()),
        )])];

        let command_window = Paragraph::new(command_window_text)
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true });

        f.render_stateful_widget(parent, main_window_rects[0], &mut parents_state);
        f.render_stateful_widget(files, main_window_rects[1], &mut files_state);
        f.render_widget(preview, main_window_rects[2]);
        f.render_widget(command_window, vertical_rects[1]);
    })?;

    execute!(stdout())?;

    Ok(())
}
