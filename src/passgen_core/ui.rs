use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use super::app::{App, InputField};

/// Main render function
pub fn render(
    f: &mut Frame,
    app: &App,
    show_master_prompt: bool,
    master_input: &str,
    custom_prompt: Option<&str>,
) {
    let size = f.area();

    if show_master_prompt {
        render_master_password_prompt(f, master_input, size, custom_prompt);
        return;
    }

    let main_area = centered_rect(60, 80, size);

    let main_block = Block::default()
        .title(" 🔐 Password Generator ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    f.render_widget(Clear, main_area);
    f.render_widget(main_block.clone(), main_area);

    let inner = main_block.inner(main_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Name input
            Constraint::Length(3), // Length input
            Constraint::Length(3), // Toggles row
            Constraint::Length(3), // Generate button
            Constraint::Length(5), // Result
            Constraint::Length(2), // Status message
            Constraint::Min(1),    // Help
        ])
        .split(inner);

    // Name input
    render_text_input(
        f,
        "Password Name",
        &app.name_input,
        app.active_field == InputField::Name,
        chunks[0],
    );

    // Length input
    render_text_input(
        f,
        "Length",
        &app.length_input,
        app.active_field == InputField::Length,
        chunks[1],
    );

    // Toggles row
    render_toggles(f, app, chunks[2]);

    // Generate button
    render_button(
        f,
        "[ Generate & Save ]",
        app.active_field == InputField::Generate,
        chunks[3],
    );

    // Result
    render_result(f, app, chunks[4]);

    // Status message
    render_status(f, app, chunks[5]);

    // Help
    render_help(f, chunks[6]);
}

fn render_master_password_prompt(
    f: &mut Frame,
    input: &str,
    size: Rect,
    custom_prompt: Option<&str>,
) {
    let area = centered_rect(50, 30, size);

    let block = Block::default()
        .title(" 🔑 Master Password ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    f.render_widget(Clear, area);
    f.render_widget(block.clone(), area);

    let inner = block.inner(area);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(3),
            Constraint::Min(1),
        ])
        .split(inner);

    let prompt_text = custom_prompt.unwrap_or("Enter master password to encrypt your vault:");
    let hint = Paragraph::new(prompt_text)
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);
    f.render_widget(hint, chunks[0]);

    // Show asterisks for password
    let masked: String = "*".repeat(input.len());
    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));
    let input_para = Paragraph::new(masked)
        .style(Style::default().fg(Color::White))
        .block(input_block);
    f.render_widget(input_para, chunks[1]);

    let help = Paragraph::new("[Enter] Confirm  [Esc] Quit")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    f.render_widget(help, chunks[2]);
}

fn render_text_input(f: &mut Frame, label: &str, value: &str, is_active: bool, area: Rect) {
    let style = if is_active {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Gray)
    };

    let block = Block::default()
        .title(format!(" {} ", label))
        .borders(Borders::ALL)
        .border_style(style);

    let cursor = if is_active { "▌" } else { "" };
    let display = format!("{}{}", value, cursor);

    let paragraph = Paragraph::new(display)
        .style(Style::default().fg(Color::White))
        .block(block);

    f.render_widget(paragraph, area);
}

fn render_toggles(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
        ])
        .split(area);

    render_toggle(
        f,
        "Special !@#",
        app.use_special,
        app.active_field == InputField::ToggleSpecial,
        chunks[0],
    );
    render_toggle(
        f,
        "Letters A-z",
        app.use_letters,
        app.active_field == InputField::ToggleLetters,
        chunks[1],
    );
    render_toggle(
        f,
        "Numbers 0-9",
        app.use_numbers,
        app.active_field == InputField::ToggleNumbers,
        chunks[2],
    );
}

fn render_toggle(f: &mut Frame, label: &str, enabled: bool, is_active: bool, area: Rect) {
    let border_style = if is_active {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Gray)
    };

    let (icon, color) = if enabled {
        ("✓", Color::Green)
    } else {
        ("✗", Color::Red)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style);

    let text = Line::from(vec![
        Span::styled(format!(" {} ", icon), Style::default().fg(color)),
        Span::raw(label),
    ]);

    let paragraph = Paragraph::new(text)
        .alignment(Alignment::Center)
        .block(block);

    f.render_widget(paragraph, area);
}

fn render_button(f: &mut Frame, label: &str, is_active: bool, area: Rect) {
    let style = if is_active {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Green)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Green)
    };

    let paragraph = Paragraph::new(label)
        .style(style)
        .alignment(Alignment::Center);

    f.render_widget(paragraph, area);
}

fn render_result(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Generated Password ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta));

    let content = if let Some(ref err) = app.error {
        Paragraph::new(err.as_str())
            .style(Style::default().fg(Color::Red))
            .alignment(Alignment::Center)
            .block(block)
    } else if let Some(ref pwd) = app.generated_password {
        // Truncate display if too long
        let display = if pwd.len() > 40 {
            format!("{}...", &pwd[..40])
        } else {
            pwd.clone()
        };
        Paragraph::new(display)
            .style(
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center)
            .block(block)
    } else {
        Paragraph::new("—")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center)
            .block(block)
    };

    f.render_widget(content, area);
}

fn render_status(f: &mut Frame, app: &App, area: Rect) {
    if let Some(ref msg) = app.status_message {
        let paragraph = Paragraph::new(msg.as_str())
            .style(Style::default().fg(Color::Cyan))
            .alignment(Alignment::Center);
        f.render_widget(paragraph, area);
    }
}

fn render_help(f: &mut Frame, area: Rect) {
    let help = Line::from(vec![
        Span::styled("[Tab/↑↓]", Style::default().fg(Color::Cyan)),
        Span::raw(" Nav  "),
        Span::styled("[Space]", Style::default().fg(Color::Cyan)),
        Span::raw(" Toggle  "),
        Span::styled("[Enter]", Style::default().fg(Color::Cyan)),
        Span::raw(" Gen  "),
        Span::styled("[v]", Style::default().fg(Color::Cyan)),
        Span::raw(" View  "),
        Span::styled("[c]", Style::default().fg(Color::Cyan)),
        Span::raw(" ChgPwd  "),
        Span::styled("[q]", Style::default().fg(Color::Cyan)),
        Span::raw(" Quit"),
    ]);
    let paragraph = Paragraph::new(help).alignment(Alignment::Center);
    f.render_widget(paragraph, area);
}

/// Render the password list viewer
pub fn render_password_list(
    f: &mut Frame,
    entries: &[super::storage::PasswordEntry],
    selected: usize,
    revealed: &std::collections::HashSet<usize>,
    mode: &super::app::ViewMode,
    status_message: Option<&str>,
    edit_buffer: &str,
) {
    let size = f.area();
    let main_area = centered_rect(70, 80, size);

    let main_block = Block::default()
        .title(" 📋 Saved Passwords ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    f.render_widget(Clear, main_area);
    f.render_widget(main_block.clone(), main_area);

    let inner = main_block.inner(main_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Min(3),    // Password list
            Constraint::Length(2), // Status / edit area
            Constraint::Length(2), // Help
        ])
        .split(inner);

    // Password list
    if entries.is_empty() {
        let empty = Paragraph::new("No passwords saved yet")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        f.render_widget(empty, chunks[0]);
    } else {
        let list_area = chunks[0];
        let visible_height = list_area.height as usize;

        // Calculate scroll offset to keep selected item visible
        let scroll_offset = if selected >= visible_height {
            selected - visible_height + 1
        } else {
            0
        };

        let mut lines: Vec<Line> = Vec::new();

        for (i, entry) in entries
            .iter()
            .enumerate()
            .skip(scroll_offset)
            .take(visible_height)
        {
            let is_selected = i == selected;
            let is_revealed = revealed.contains(&i);

            let prefix = if is_selected { "▸ " } else { "  " };

            // Show edit buffer when editing
            let (name_display, password_display) = if is_selected {
                match mode {
                    super::app::ViewMode::EditName => {
                        (format!("{}▌", edit_buffer), "••••••••••••".to_string())
                    }
                    super::app::ViewMode::EditPassword => {
                        (entry.name.clone(), format!("{}▌", edit_buffer))
                    }
                    _ => {
                        let pwd = if is_revealed {
                            entry.password.clone()
                        } else {
                            "••••••••••••".to_string()
                        };
                        (entry.name.clone(), pwd)
                    }
                }
            } else {
                let pwd = if is_revealed {
                    entry.password.clone()
                } else {
                    "••••••••••••".to_string()
                };
                (entry.name.clone(), pwd)
            };

            let name_style = if is_selected {
                if *mode == super::app::ViewMode::EditName {
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                }
            } else {
                Style::default().fg(Color::White)
            };

            let pwd_style = if is_selected && *mode == super::app::ViewMode::EditPassword {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else if is_revealed {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            let line = Line::from(vec![
                Span::styled(prefix, Style::default().fg(Color::Yellow)),
                Span::styled(format!("{:<20}", name_display), name_style),
                Span::raw(" → "),
                Span::styled(password_display, pwd_style),
            ]);
            lines.push(line);
        }

        let list = Paragraph::new(lines);
        f.render_widget(list, chunks[0]);
    }

    // Status / confirm area
    let status_content = match mode {
        super::app::ViewMode::ConfirmDelete => {
            let name = entries.get(selected).map(|e| e.name.as_str()).unwrap_or("");
            Line::from(vec![
                Span::styled("Delete '", Style::default().fg(Color::Red)),
                Span::styled(name, Style::default().fg(Color::Yellow)),
                Span::styled("'? ", Style::default().fg(Color::Red)),
                Span::styled("[y]", Style::default().fg(Color::Green)),
                Span::raw("es / "),
                Span::styled("[n]", Style::default().fg(Color::Red)),
                Span::raw("o"),
            ])
        }
        super::app::ViewMode::EditName => Line::from(vec![
            Span::styled("Editing name", Style::default().fg(Color::Green)),
            Span::raw(" — Press "),
            Span::styled("[Enter]", Style::default().fg(Color::Cyan)),
            Span::raw(" to save, "),
            Span::styled("[Esc]", Style::default().fg(Color::Cyan)),
            Span::raw(" to cancel"),
        ]),
        super::app::ViewMode::EditPassword => Line::from(vec![
            Span::styled("Editing password", Style::default().fg(Color::Green)),
            Span::raw(" — Press "),
            Span::styled("[Enter]", Style::default().fg(Color::Cyan)),
            Span::raw(" to save, "),
            Span::styled("[Esc]", Style::default().fg(Color::Cyan)),
            Span::raw(" to cancel"),
        ]),
        super::app::ViewMode::Browse => {
            if let Some(msg) = status_message {
                Line::from(Span::styled(msg, Style::default().fg(Color::Cyan)))
            } else {
                Line::from("")
            }
        }
    };
    let status_para = Paragraph::new(status_content).alignment(Alignment::Center);
    f.render_widget(status_para, chunks[1]);

    // Help bar for viewer (context-sensitive)
    let help = match mode {
        super::app::ViewMode::Browse => Line::from(vec![
            Span::styled("[↑↓]", Style::default().fg(Color::Cyan)),
            Span::raw(" Nav "),
            Span::styled("[Space]", Style::default().fg(Color::Cyan)),
            Span::raw(" Reveal "),
            Span::styled("[y]", Style::default().fg(Color::Cyan)),
            Span::raw(" Copy "),
            Span::styled("[e]", Style::default().fg(Color::Cyan)),
            Span::raw(" EditName "),
            Span::styled("[p]", Style::default().fg(Color::Cyan)),
            Span::raw(" EditPwd "),
            Span::styled("[d]", Style::default().fg(Color::Cyan)),
            Span::raw(" Del "),
            Span::styled("[Esc]", Style::default().fg(Color::Cyan)),
            Span::raw(" Back"),
        ]),
        _ => Line::from(vec![
            Span::styled("[Esc]", Style::default().fg(Color::Cyan)),
            Span::raw(" Cancel"),
        ]),
    };
    let help_para = Paragraph::new(help).alignment(Alignment::Center);
    f.render_widget(help_para, chunks[2]);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}
