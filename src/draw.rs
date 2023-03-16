use crossterm::terminal::enable_raw_mode;
use crossterm::{event::EnableMouseCapture, execute, terminal::EnterAlternateScreen};
use std::collections::HashSet;
use std::io::Stdout;
use std::io::{self, stdout};
use tui::layout::{Alignment, Constraint, Layout};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Cell, List, ListItem, ListState, Paragraph, Row, Table, TableState, Wrap};
use tui::{
    backend::CrosstermBackend,
    widgets::{Block, Borders},
    Terminal,
};

use crate::error::FilmanError;
use crate::path::Path;
use crate::state::Mode;
use crate::state::State;

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct DirectoryEntry {
    name: String,
    info: String,
}

// TODO: Don't leak things like filenames, selected or error_message into this interface
pub struct RenderState<'a> {
    pub files_in_pwd: Vec<DirectoryEntry>,
    pub selected_in_pwd: Option<usize>,

    pub multi_select: HashSet<String>,
    pub yanked: HashSet<String>,

    pub files_in_parent: Vec<String>,
    pub selected_in_parent: Option<usize>,

    pub preview: &'a str,

    pub command: Option<String>,
    pub error_message: Option<&'a str>,
}

impl<'a> TryFrom<&'a State> for RenderState<'a> {
    type Error = FilmanError;

    fn try_from(other: &'a State) -> Result<Self, FilmanError> {
        let command = match &other.mode {
            Mode::NormalMode => None,
            Mode::CommandMode(pr) => Some(pr.result().to_string()),
            Mode::ShellCommandMode(pr) => Some(pr.result().to_string()),
        };

        let files_in_pwd = other
            .files_in_pwd()?
            .iter()
            .map(|x| {
                Ok(DirectoryEntry {
                    name: x.filename()?.to_string(),
                    info: human_bytes::human_bytes(x.size().unwrap_or(0) as f64),
                })
            })
            .collect::<Result<Vec<_>, FilmanError>>()?;

        let selected_in_pwd = Some(other.selected_index_in_pwd());

        let files_in_parent = other
            .files_in_parent()?
            .iter()
            .map(|x| x.file_name().unwrap().to_str().unwrap().to_string())
            .collect();
        let selected_in_parent = other.selected_index_in_parent()?;

        let multi_select = other
            .multi_select
            .iter()
            .filter(|p| p.parent() == Some(&other.pwd))
            .map(|p| p.file_name().unwrap().to_str().unwrap().to_string())
            .collect();

        let yanked = other
            .yanked
            .iter()
            .filter(|p| p.parent() == Some(&other.pwd))
            .map(|p| p.file_name().unwrap().to_str().unwrap().to_string())
            .collect();

        let preview = other.file_contents.as_deref().unwrap_or("Binary file");
        let error_message = other.error_message.as_deref();

        Ok(RenderState {
            yanked,
            files_in_pwd,
            selected_in_pwd,
            files_in_parent,
            selected_in_parent,
            command,
            multi_select,
            preview,
            error_message,
        })
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

fn format_file(
    file: &DirectoryEntry,
    selected: bool,
    yanked: bool,
    multi_selected: bool,
) -> String {
    let mut ret = String::new();

    if selected {
        return format!(">> {}", file.name);
    }

    if yanked {
        ret += "Y";
    } else {
        ret += " ";
    }

    if multi_selected {
        ret += "S";
    } else {
        ret += " ";
    }
    ret += " ";
    ret += &file.name;
    ret
}

pub fn draw(
    state: &RenderState,
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
) -> Result<(), io::Error> {
    terminal.draw(|f| {
        // Layout
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

        // Parent list
        let parent_list_items: Vec<ListItem> = state
            .files_in_parent
            .iter()
            .map(|x| ListItem::new(x.to_string()))
            .collect();

        let parent = List::new(parent_list_items)
            .block(Block::default().title("Parent").borders(Borders::ALL))
            .highlight_style(Style::default().add_modifier(Modifier::BOLD));

        // Preview window
        let preview = Paragraph::new(state.preview)
            .block(Block::default().title("Preview").borders(Borders::ALL))
            .wrap(Wrap { trim: false });

        // Files table
        let table = Table::new(
            state
                .files_in_pwd
                .clone()
                .into_iter()
                .map(|x| {
                    let formatted = format_file(
                        &x,
                        false,
                        state.yanked.contains(&x.name),
                        state.multi_select.contains(&x.name),
                    );

                    Row::new(vec![Cell::from(formatted), Cell::from(x.info)])
                        .style(Style::default())
                })
                .collect::<Vec<_>>(),
        )
        .style(Style::default().fg(Color::White))
        .block(Block::default().title("Files").borders(Borders::ALL))
        .widths(&[Constraint::Percentage(80), Constraint::Percentage(20)])
        .column_spacing(1)
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol(">>");

        // Select state for list and table
        let mut files_state = TableState::default();
        files_state.select(state.selected_in_pwd);

        let mut parents_state = ListState::default();
        parents_state.select(state.selected_in_parent);

        // Command window
        let command_window_string = state
            .error_message
            .unwrap_or(state.command.as_deref().unwrap_or(""));

        let command_window_text = vec![Spans::from(vec![Span::raw(command_window_string)])];
        let command_window = Paragraph::new(command_window_text)
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true });

        // Add to window
        f.render_stateful_widget(parent, main_window_rects[0], &mut parents_state);
        f.render_stateful_widget(table, main_window_rects[1], &mut files_state);
        f.render_widget(preview, main_window_rects[2]);
        f.render_widget(command_window, vertical_rects[1]);
    })?;

    execute!(stdout())?;

    Ok(())
}
