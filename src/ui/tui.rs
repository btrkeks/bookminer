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
use ratatui::prelude::{Alignment, Color, Constraint, Direction, Layout, Line, Modifier, Rect, Span, Style, Stylize, Text};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap};
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

    // pub fn mouse(mut self, mouse: bool) -> Self {
    //     self.mouse = mouse;
    //     self
    // }

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
        Ok(())
    }

    pub fn resume(&mut self) -> Result<()> {
        self.enter()?;
        self.terminal.clear()?;
        Ok(())
    }

    // pub fn force_refresh(&mut self) -> Result<()> {
    //     self.terminal.clear()?;
    //     self.terminal.flush()?;
    //     Ok(())
    // }

    pub(crate) fn edit_file(&mut self, file_path: &Path) -> Result<()> {
        self.suspend()?;
        edit_file(file_path)?;
        self.resume()?;
        Ok(())
    }

    pub fn show_dialog(&mut self, msg: &str) -> Result<bool> {
        let mut selected = true;

        let normal_style = Style::default().white();
        let yes_style = Style::default().green();
        let no_style = Style::default().red();
        let block_style = Style::default();

        loop {
            self.terminal.draw(|f| {
                let size = f.area();

                // Calculate required dimensions
                let msg_width = size.width.saturating_sub(4);
                let content_height = 4; // title + message + space + options
                let content_width = msg.len().max(20).min(msg_width as usize) as u16 + 4;

                let area = centered_content_rect(content_width, content_height, size);

                let block = Block::new()
                    .title("Confirmation")
                    .borders(Borders::ALL)
                    .style(block_style);

                f.render_widget(Clear, area);
                f.render_widget(block.clone(), area);

                let inner_area = block.inner(area);
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(1),  // Message
                        Constraint::Length(1),  // Space
                        Constraint::Length(1),  // Options
                    ])
                    .split(inner_area);

                let message = Paragraph::new(msg)
                    .style(normal_style)
                    .wrap(Wrap { trim: true });
                f.render_widget(message, chunks[0]);

                let options = Line::from(vec![
                    Span::styled("Yes", if selected { yes_style } else { normal_style }),
                    Span::raw(" / "),
                    Span::styled("No", if !selected { no_style } else { normal_style }),
                ]);

                let options = Paragraph::new(options)
                    .alignment(Alignment::Center);
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

        // Helper function to advance selection
        fn advance_selection(current: Option<usize>, max: usize) -> Option<usize> {
            if max == 0 {
                return None;
            }
            Some(match current {
                Some(i) => if i >= max - 1 { 0 } else { i + 1 },
                None => 0,
            })
        }

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
                    "i: add tag, Space: toggle and advance, d: delete, g/G: first/last, Enter: confirm"
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
                            if !new_tag.is_empty() {
                                tags.push(new_tag.clone());
                                selected_tags.push(true);
                                new_tag.clear();
                            }
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
                        KeyCode::Char('g') => {
                            if !tags.is_empty() {
                                list_state.select(Some(0)); // Jump to first
                            }
                        }
                        KeyCode::Char('G') => {
                            if !tags.is_empty() {
                                list_state.select(Some(tags.len() - 1)); // Jump to last
                            }
                        }
                        KeyCode::Char('d') => {
                            if let Some(selected_idx) = list_state.selected() {
                                if !tags.is_empty() {
                                    // Remove the tag and its selection state
                                    tags.remove(selected_idx);
                                    selected_tags.remove(selected_idx);

                                    // Adjust the selection to prevent out-of-bounds
                                    if tags.is_empty() {
                                        list_state.select(None);
                                    } else if selected_idx >= tags.len() {
                                        list_state.select(Some(tags.len() - 1));
                                    }
                                }
                            }
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            if !tags.is_empty() {
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
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            if !tags.is_empty() {
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
                        }
                        KeyCode::Char(' ') => {
                            if let Some(i) = list_state.selected() {
                                // Toggle the current selection
                                selected_tags[i] = !selected_tags[i];
                                // Advance the selection (wrapping around)
                                list_state.select(advance_selection(Some(i), tags.len()));
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

fn centered_content_rect(width: u16, height: u16, container: Rect) -> Rect {
    let x = container.x + (container.width.saturating_sub(width)) / 2;
    let y = container.y + (container.height.saturating_sub(height)) / 2;

    Rect {
        x,
        y,
        width: width.min(container.width),
        height: height.min(container.height),
    }
}

fn edit_file(file_path: &Path) -> io::Result<()> {
    if !file_path.exists() {
        fs::File::create(file_path)?;
    }

    let editor_name = get_editor_binary_name();
    Command::new(editor_name).arg(file_path).status()?;
    Ok(())
}
