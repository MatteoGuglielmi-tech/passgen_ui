use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use super::app::{App, InputField};

/// Main render function
pub fn render(f: &mut Frame, app: &App, show_master_prompt: bool, master_input: &str) {
    let size = f.area();

    if show_master_prompt {
        render_master_password_prompt(f, master_input, size);
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

fn render_master_password_prompt(f: &mut Frame, input: &str, size: Rect) {
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

    let hint = Paragraph::new("Enter master password to encrypt your vault:")
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
        Span::raw(" Navigate  "),
        Span::styled("[Space]", Style::default().fg(Color::Cyan)),
        Span::raw(" Toggle  "),
        Span::styled("[Enter]", Style::default().fg(Color::Cyan)),
        Span::raw(" Generate  "),
        Span::styled("[q]", Style::default().fg(Color::Cyan)),
        Span::raw(" Quit"),
    ]);
    let paragraph = Paragraph::new(help).alignment(Alignment::Center);
    f.render_widget(paragraph, area);
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
