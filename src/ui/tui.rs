use crate::env_variables::get_editor_binary_name;
use anyhow::Result;
use crossterm::event;
use crossterm::event::{Event, KeyCode};
use ratatui::backend::CrosstermBackend as Backend;
use ratatui::crossterm::{
    cursor,
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::{
    Color, Constraint, Direction, Layout, Line, Modifier, Rect, Span, Style, Text,
};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph};
use std::path::Path;
use std::process::Command;
use std::{
    fs, io,
    ops::{Deref, DerefMut},
};

pub struct Tui {
    pub terminal: ratatui::Terminal<Backend<std::io::Stderr>>,
    pub mouse: bool,
}

impl Tui {
    pub fn new() -> Result<Self> {
        let terminal = ratatui::Terminal::new(Backend::new(std::io::stderr()))?;
        let mouse = false;
        Ok(Self { terminal, mouse })
    }

    pub fn mouse(mut self, mouse: bool) -> Self {
        self.mouse = mouse;
        self
    }

    pub fn enter(&mut self) -> Result<()> {
        crossterm::terminal::enable_raw_mode()?;
        crossterm::execute!(std::io::stderr(), EnterAlternateScreen, cursor::Hide)?;
        if self.mouse {
            crossterm::execute!(std::io::stderr(), EnableMouseCapture)?;
        }
        Ok(())
    }

    pub fn exit(&mut self) -> Result<()> {
        if crossterm::terminal::is_raw_mode_enabled()? {
            self.flush()?;
            if self.mouse {
                crossterm::execute!(std::io::stderr(), DisableMouseCapture)?;
            }
            crossterm::execute!(std::io::stderr(), LeaveAlternateScreen, cursor::Show)?;
            crossterm::terminal::disable_raw_mode()?;
        }
        Ok(())
    }

    pub fn suspend(&mut self) -> Result<()> {
        self.exit()?;
        // #[cfg(not(windows))]
        // signal_hook::low_level::raise(signal_hook::consts::signal::SIGTSTP)?;
        Ok(())
    }

    pub fn resume(&mut self) -> Result<()> {
        self.enter()?;
        Ok(())
    }

    pub(crate) fn edit_file(&mut self, file_path: &Path) -> Result<()> {
        self.suspend()?;
        edit_file(file_path)?;
        self.resume()?;
        Ok(())
    }

    pub fn show_dialog(&mut self, msg: &str) -> Result<bool> {
        let mut selected = true;

        loop {
            self.terminal.draw(|f| {
                let size = f.area();
                let block = Block::default().title("Confirmation").borders(Borders::ALL);

                let area = centered_rect(60, 20, size);

                f.render_widget(Clear, area); // Clear the area first
                f.render_widget(block, area);

                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(2)
                    .constraints(
                        [
                            Constraint::Length(1),
                            Constraint::Length(1),
                            Constraint::Length(1),
                        ]
                        .as_ref(),
                    )
                    .split(area);

                let text = vec![Span::raw(msg)];
                let text_line = Line::from(text);
                let message = Paragraph::new(Text::from(text_line))
                    .wrap(ratatui::widgets::Wrap { trim: true });
                f.render_widget(message, chunks[0]);

                let yes = Span::styled(
                    "Yes",
                    Style::default().fg(if selected { Color::Green } else { Color::White }),
                );
                let no = Span::styled(
                    "No",
                    Style::default().fg(if !selected { Color::Red } else { Color::White }),
                );
                let options = Paragraph::new(Line::from(vec![yes, Span::raw(" / "), no]));
                f.render_widget(options, chunks[2]);
            })?;

            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Left | KeyCode::Right => {
                        selected = !selected;
                    }
                    KeyCode::Enter => {
                        return Ok(selected);
                    }
                    KeyCode::Char('y') | KeyCode::Char('Y') => {
                        return Ok(true);
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') => {
                        return Ok(false);
                    }
                    KeyCode::Esc => {
                        return Ok(false);
                    }
                    _ => {}
                }
            }
        }
    }

    pub fn show_single_selection_menu<T: AsRef<str>>(
        &mut self,
        title: &str,
        items: &[T],
    ) -> anyhow::Result<usize> {
        let mut list_state = ListState::default();
        list_state.select(Some(0));

        loop {
            self.terminal.draw(|f| {
                let size = f.area();
                let block = Block::default().title(title).borders(Borders::ALL);
                let items: Vec<ListItem> =
                    items.iter().map(|i| ListItem::new(i.as_ref())).collect();
                let list = List::new(items).block(block).highlight_style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                );

                f.render_stateful_widget(list, size, &mut list_state);
            })?;

            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Down | KeyCode::Char('j') => {
                        let i = match list_state.selected() {
                            Some(i) => {
                                if i >= items.len() - 1 {
                                    0
                                } else {
                                    i + 1
                                }
                            }
                            None => 0,
                        };
                        list_state.select(Some(i));
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        let i = match list_state.selected() {
                            Some(i) => {
                                if i == 0 {
                                    items.len() - 1
                                } else {
                                    i - 1
                                }
                            }
                            None => 0,
                        };
                        list_state.select(Some(i));
                    }
                    KeyCode::Enter => {
                        let selected = list_state.selected().unwrap_or(0);
                        return Ok(selected);
                    }
                    _ => {}
                }
            }
        }
    }

    pub fn show_tag_menu(&mut self, tags: &mut Vec<String>) -> anyhow::Result<Vec<String>> {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        let mut selected_tags: Vec<bool> = vec![false; tags.len()];
        let mut new_tag = String::new();
        let mut input_mode = false;

        loop {
            self.terminal.draw(|f| {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(1)
                    .constraints(
                        [
                            Constraint::Length(1),
                            Constraint::Min(1),
                            Constraint::Length(3),
                        ]
                        .as_ref(),
                    )
                    .split(f.area());

                let help_message = if input_mode {
                    "Esc: stop editing, Enter: record tag"
                } else {
                    "i: add tag, Space: toggle, Enter: confirm"
                };
                let help_paragraph = Paragraph::new(help_message)
                    .style(Style::default().fg(Color::Gray))
                    .alignment(ratatui::layout::Alignment::Center);
                f.render_widget(help_paragraph, chunks[0]);

                let items: Vec<ListItem> = tags
                    .iter()
                    .enumerate()
                    .map(|(i, t)| {
                        let content = Line::from(Span::raw(format!(
                            "{} {}",
                            if selected_tags[i] { "[x]" } else { "[ ]" },
                            t
                        )));
                        ListItem::new(content)
                    })
                    .collect();

                let items = List::new(items)
                    .block(Block::default().borders(Borders::ALL).title("Tags"))
                    .highlight_style(
                        Style::default()
                            .bg(Color::Yellow)
                            .fg(Color::Black)
                            .add_modifier(Modifier::BOLD),
                    );

                f.render_stateful_widget(items, chunks[1], &mut list_state);

                let input = Paragraph::new(new_tag.as_str())
                    .style(match input_mode {
                        true => Style::default().fg(Color::Yellow),
                        false => Style::default(),
                    })
                    .block(Block::default().borders(Borders::ALL).title("New Tag"));
                f.render_widget(input, chunks[2]);
            })?;

            if let Event::Key(key) = event::read()? {
                if input_mode {
                    match key.code {
                        KeyCode::Esc => {
                            input_mode = false;
                        }
                        KeyCode::Char(c) => {
                            new_tag.push(c);
                        }
                        KeyCode::Backspace => {
                            new_tag.pop();
                        }
                        KeyCode::Enter => {
                            tags.push(new_tag.clone());
                            selected_tags.push(true);
                            new_tag.clear();
                            input_mode = false;
                        }
                        _ => {}
                    }
                } else {
                    match key.code {
                        KeyCode::Enter => {
                            return Ok(tags
                                .iter()
                                .enumerate()
                                .filter(|&(i, _)| selected_tags[i])
                                .map(|(_, tag)| tag.clone())
                                .collect());
                        }
                        KeyCode::Char('i') => {
                            input_mode = true;
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            let i = match list_state.selected() {
                                Some(i) => {
                                    if i >= tags.len() - 1 {
                                        0
                                    } else {
                                        i + 1
                                    }
                                }
                                None => 0,
                            };
                            list_state.select(Some(i));
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            let i = match list_state.selected() {
                                Some(i) => {
                                    if i == 0 {
                                        tags.len() - 1
                                    } else {
                                        i - 1
                                    }
                                }
                                None => 0,
                            };
                            list_state.select(Some(i));
                        }
                        KeyCode::Char(' ') => {
                            if let Some(i) = list_state.selected() {
                                selected_tags[i] = !selected_tags[i];
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

impl Deref for Tui {
    type Target = ratatui::Terminal<Backend<std::io::Stderr>>;

    fn deref(&self) -> &Self::Target {
        &self.terminal
    }
}

impl DerefMut for Tui {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.terminal
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        self.exit().unwrap();
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}

// fn draw_tag_menu<B: Backend>(
//     f: &mut Frame<B>,
//     chunks: &[Rect],
//     input_mode: bool,
//     new_tag: &str,
//     tags: &[String],
//     selected_tags: &[bool],
//     list_state: &mut ListState,
// ) {
//     // Draw help message
//     let (msg, style) = create_help_message(input_mode);
//     let help_message = Paragraph::new(Line::from(msg)).style(style);
//     f.render_widget(help_message, chunks[0]);
//
//     // Draw tag list
//     let items: Vec<ListItem> = create_tag_list_items(tags, selected_tags);
//     let items = List::new(items)
//         .block(Block::default().borders(Borders::ALL).title("Tags"))
//         .highlight_style(
//             Style::default()
//                 .bg(Color::Yellow)
//                 .fg(Color::Black)
//                 .add_modifier(Modifier::BOLD),
//         );
//     f.render_stateful_widget(items, chunks[1], list_state);
//
//     // Draw input field
//     let input = Paragraph::new(new_tag)
//         .style(if input_mode {
//             Style::default().fg(Color::Yellow)
//         } else {
//             Style::default()
//         })
//         .block(Block::default().borders(Borders::ALL).title("New Tag"));
//     f.render_widget(input, chunks[2]);
// }
//
// fn create_help_message(input_mode: bool) -> (Vec<Span<'static>>, Style) {
//     if input_mode {
//         (
//             vec![
//                 Span::raw("Press "),
//                 Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
//                 Span::raw(" to stop editing, "),
//                 Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
//                 Span::raw(" to record the new tag"),
//             ],
//             Style::default(),
//         )
//     } else {
//         (
//             vec![
//                 Span::raw("Press "),
//                 Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
//                 Span::raw(" to exit, "),
//                 Span::styled("i", Style::default().add_modifier(Modifier::BOLD)),
//                 Span::raw(" to add new tag"),
//             ],
//             Style::default(),
//         )
//     }
// }
//
// fn create_tag_list_items(tags: &[String], selected_tags: &[bool]) -> Vec<ListItem> {
//     tags.iter()
//         .enumerate()
//         .map(|(i, t)| {
//             let content = Line::from(Span::raw(format!(
//                 "{} {}",
//                 if selected_tags[i] { "[x]" } else { "[ ]" },
//                 t
//             )));
//             ListItem::new(content)
//         })
//         .collect()
// }

fn edit_file(file_path: &Path) -> io::Result<()> {
    if !file_path.exists() {
        fs::File::create(file_path)?;
    }

    let editor_name = get_editor_binary_name();
    Command::new(editor_name).arg(file_path).status()?;
    Ok(())
}
