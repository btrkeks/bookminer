use std::{fs, io};
use std::path::{PathBuf};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    style::{Color, Modifier, Style},
    Terminal,
};
use crate::menu_actions::{EditBackAction, EditFrontAction, MenuAction, SendCardAction};
use crate::paths::get_tags_file;
use anyhow::{Context, Result};
use crate::edit_files::{edit_back, edit_front};
use crate::menu_actions::CancelAction;

fn show_tags() -> Result<Vec<String>> {
    let mut tags = load_tags()?;

    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;

    let mut selected_tags = vec![false; tags.len()];
    let mut current_index = 0;
    let mut input = String::new();
    let mut input_mode = false;

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(3), Constraint::Length(3)].as_ref())
                .split(f.size());

            let items: Vec<ListItem> = tags
                .iter()
                .enumerate()
                .map(|(i, t)| {
                    let checkbox = if selected_tags[i] { "[x]" } else { "[ ]" };
                    let content = format!("{} {}", checkbox, t);
                    let style = if i == current_index && !input_mode {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default()
                    };
                    ListItem::new(content).style(style)
                })
                .collect();

            let tags_list = List::new(items)
                .block(Block::default().title("Tags (Space to select, Enter to finish, i to add new)").borders(Borders::ALL))
                .highlight_style(Style::default().add_modifier(Modifier::BOLD));

            f.render_widget(tags_list, chunks[0]);

            let input_block = Paragraph::new(input.as_ref())
                .style(Style::default().fg(if input_mode { Color::Yellow } else { Color::White }))
                .block(Block::default().title("New Tag").borders(Borders::ALL));
            f.render_widget(input_block, chunks[1]);
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') if !input_mode => break,
                KeyCode::Char('i') if !input_mode => input_mode = true,
                KeyCode::Esc if input_mode => input_mode = false,
                KeyCode::Enter if input_mode => {
                    if !input.is_empty() {
                        tags.push(input.clone());
                        selected_tags.push(true);  // Automatically select new tag
                        current_index = tags.len() - 1;
                        input.clear();
                    }
                    input_mode = false;
                }
                KeyCode::Char(c) if input_mode => input.push(c),
                KeyCode::Backspace if input_mode => { input.pop(); }
                KeyCode::Char('k') | KeyCode::Up if !input_mode => {
                    current_index = current_index.saturating_sub(1);
                }
                KeyCode::Char('j') | KeyCode::Down if !input_mode => {
                    current_index = (current_index + 1).min(tags.len() - 1);
                }
                KeyCode::Char(' ') if !input_mode => {
                    selected_tags[current_index] = !selected_tags[current_index];
                }
                KeyCode::Enter if !input_mode => {
                    break;
                }
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    save_tags(&tags);

    let selected = tags.iter().enumerate()
        .filter_map(|(i, tag)| if selected_tags[i] { Some(tag.clone()) } else { None })
        .collect();

    Ok(selected)
}


fn save_tags(tags: &Vec<String>) -> Result<()> {
    let tags_file = get_tags_file()?;
    fs::write(&tags_file, tags.join("\n")).context("Storing tags")
}

fn load_tags() -> Result<Vec<String>> {
    let tags_file = get_tags_file()?;

    let tags = if tags_file.exists() {
        fs::read_to_string(&tags_file)?
            .lines()
            .map(String::from)
            .collect::<Vec<String>>()
    } else {
        Vec::new()
    };

    Ok(tags)
}

fn show_menu() -> Result<Box<dyn MenuAction>> {
    let menu_items = vec!["Send Card", "Edit Front", "Edit Back", "Cancel"];
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;

    let mut selected = 0;

    loop {
        terminal.draw(|f| {
            let items: Vec<ListItem> = menu_items
                .iter()
                .enumerate()
                .map(|(i, &item)| {
                    let style = if i == selected {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default()
                    };
                    ListItem::new(item).style(style)
                })
                .collect();

            let menu_list = List::new(items)
                .block(Block::default().title("Menu").borders(Borders::ALL))
                .highlight_style(Style::default().add_modifier(Modifier::BOLD));

            f.render_widget(menu_list, f.size());
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('k') | KeyCode::Up => {
                    selected = (selected + menu_items.len() - 1) % menu_items.len();
                }
                KeyCode::Char('j') | KeyCode::Down => {
                    selected = (selected + 1) % menu_items.len();
                }
                KeyCode::Enter => break,
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(match selected {
        0 => Box::new(SendCardAction::new()),
        1 => Box::new(EditFrontAction::new()),
        2 => Box::new(EditBackAction::new()),
        _ => Box::new(CancelAction::new()),
    })
}

pub fn run_main_application(tmp_dir: PathBuf, screenshot_path: PathBuf) -> Result<()> {
    edit_front(&tmp_dir)?;
    edit_back(&tmp_dir)?;
    show_tags()?;

    while let chosen_action = show_menu()? {
        chosen_action.act(&tmp_dir, &screenshot_path)?;

        if chosen_action.should_exit() {
            break;
        }
    }

    Ok(())
}