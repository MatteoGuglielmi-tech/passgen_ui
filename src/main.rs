use arboard::Clipboard;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use passgen_ui::passgen_core::{
    app::{App, ViewMode},
    storage::{PasswordEntry, Storage},
    ui,
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;

/// Application phase
enum Phase {
    MasterPassword,
    Main,
    ChangeMasterPassword { step: ChangeStep },
    ViewPasswords { mode: ViewMode },
}

enum ChangeStep {
    EnterOld,
    EnterNew,
    ConfirmNew,
}

/// State for the password viewer
struct ViewerState {
    entries: Vec<PasswordEntry>,
    selected: usize,
    revealed: std::collections::HashSet<usize>,
    status_message: Option<String>,
    edit_buffer: String,
}

fn main() -> io::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run(&mut terminal);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = result {
        eprintln!("Error: {err:?}");
    }

    Ok(())
}

fn run(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    let mut app = App::new();
    let mut phase = Phase::MasterPassword;
    let mut master_input = String::new();
    let mut storage: Option<Storage> = None;

    // For password change flow
    let mut new_password = String::new();
    let mut confirm_password = String::new();

    // For password viewer
    let mut viewer_state: Option<ViewerState> = None;

    loop {
        // Render
        terminal.draw(|f| match &phase {
            Phase::MasterPassword => {
                ui::render(f, &app, true, &master_input, None);
            }
            Phase::Main => {
                ui::render(f, &app, false, "", None);
            }
            Phase::ChangeMasterPassword { step } => {
                let prompt = match step {
                    ChangeStep::EnterOld => ("Enter current master password:", &master_input),
                    ChangeStep::EnterNew => ("Enter NEW master password:", &new_password),
                    ChangeStep::ConfirmNew => ("Confirm NEW master password:", &confirm_password),
                };
                ui::render(f, &app, true, prompt.1, Some(prompt.0));
            }
            Phase::ViewPasswords { mode } => {
                if let Some(ref state) = viewer_state {
                    ui::render_password_list(
                        f,
                        &state.entries,
                        state.selected,
                        &state.revealed,
                        mode,
                        state.status_message.as_deref(),
                        &state.edit_buffer,
                    );
                }
            }
        })?;

        // Handle input
        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }

            match &mut phase {
                Phase::MasterPassword => match key.code {
                    KeyCode::Esc => return Ok(()),
                    KeyCode::Enter => {
                        if master_input.is_empty() {
                            continue;
                        }
                        match Storage::new(&master_input) {
                            Ok(s) => {
                                storage = Some(s);
                                phase = Phase::Main;
                                master_input.clear();
                            }
                            Err(e) => {
                                app.error = Some(e);
                                master_input.clear();
                            }
                        }
                    }
                    KeyCode::Backspace => {
                        master_input.pop();
                    }
                    KeyCode::Char(c) => {
                        master_input.push(c);
                    }
                    _ => {}
                },
                Phase::Main => {
                    match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Esc => return Ok(()),
                        KeyCode::Char('c') => {
                            // Start change password flow
                            phase = Phase::ChangeMasterPassword {
                                step: ChangeStep::EnterOld,
                            };
                            master_input.clear();
                            new_password.clear();
                            confirm_password.clear();
                            app.error = None;
                            app.status_message = None;
                        }
                        KeyCode::Char('v') => {
                            // View saved passwords
                            if let Some(ref store) = storage {
                                match store.load() {
                                    Ok(entries) => {
                                        viewer_state = Some(ViewerState {
                                            entries,
                                            selected: 0,
                                            revealed: std::collections::HashSet::new(),
                                            status_message: None,
                                            edit_buffer: String::new(),
                                        });
                                        phase = Phase::ViewPasswords { mode: ViewMode::Browse };
                                        app.error = None;
                                    }
                                    Err(e) => {
                                        app.error = Some(format!("Failed to load: {}", e));
                                    }
                                }
                            }
                        }
                        KeyCode::Tab | KeyCode::Down => app.next_field(),
                        KeyCode::BackTab | KeyCode::Up => app.prev_field(),
                        KeyCode::Enter => {
                            app.generate();
                            // Auto-save if generation succeeded
                            if app.generated_password.is_some()
                                && let Some(ref store) = storage
                                && let Some(entry) = app.get_entry()
                            {
                                match store.save(entry) {
                                    Ok(_) => {
                                        app.status_message =
                                            Some(format!("✓ Saved to {}", store.path().display()));
                                    }
                                    Err(e) => {
                                        app.error = Some(format!("Save failed: {}", e));
                                    }
                                }
                            }
                        }
                        KeyCode::Char(' ') => {
                            app.toggle_current();
                        }
                        KeyCode::Backspace => {
                            if let Some(input) = app.current_text_input() {
                                input.pop();
                            }
                        }
                        KeyCode::Char(c) => {
                            if let Some(input) = app.current_text_input() {
                                input.push(c);
                            }
                        }
                        _ => {}
                    }
                }
                Phase::ChangeMasterPassword { step } => {
                    match key.code {
                        KeyCode::Esc => {
                            // Cancel and go back to main
                            phase = Phase::Main;
                            master_input.clear();
                            new_password.clear();
                            confirm_password.clear();
                            app.error = None;
                        }
                        KeyCode::Enter => {
                            match step {
                                ChangeStep::EnterOld => {
                                    // Verify old password by trying to load
                                    match Storage::new(&master_input) {
                                        Ok(s) => {
                                            storage = Some(s);
                                            *step = ChangeStep::EnterNew;
                                            app.error = None;
                                        }
                                        Err(e) => {
                                            app.error = Some(e);
                                            master_input.clear();
                                        }
                                    }
                                }
                                ChangeStep::EnterNew => {
                                    if new_password.is_empty() {
                                        app.error = Some("Password cannot be empty".into());
                                    } else {
                                        *step = ChangeStep::ConfirmNew;
                                        app.error = None;
                                    }
                                }
                                ChangeStep::ConfirmNew => {
                                    if confirm_password != new_password {
                                        app.error = Some("Passwords don't match".into());
                                        confirm_password.clear();
                                    } else if let Some(ref store) = storage {
                                        match store.change_master_password(&new_password) {
                                            Ok(new_store) => {
                                                storage = Some(new_store);
                                                app.status_message =
                                                    Some("✓ Master password changed!".into());
                                                app.error = None;
                                                phase = Phase::Main;
                                                master_input.clear();
                                                new_password.clear();
                                                confirm_password.clear();
                                            }
                                            Err(e) => {
                                                app.error = Some(format!("Failed: {}", e));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        KeyCode::Backspace => match step {
                            ChangeStep::EnterOld => {
                                master_input.pop();
                            }
                            ChangeStep::EnterNew => {
                                new_password.pop();
                            }
                            ChangeStep::ConfirmNew => {
                                confirm_password.pop();
                            }
                        },
                        KeyCode::Char(c) => match step {
                            ChangeStep::EnterOld => master_input.push(c),
                            ChangeStep::EnterNew => new_password.push(c),
                            ChangeStep::ConfirmNew => confirm_password.push(c),
                        },
                        _ => {}
                    }
                }
                Phase::ViewPasswords { mode } => {
                    if let Some(state) = &mut viewer_state {
                        match mode {
                            ViewMode::Browse => {
                                match key.code {
                                    KeyCode::Esc | KeyCode::Char('q') => {
                                        phase = Phase::Main;
                                        viewer_state = None;
                                    }
                                    KeyCode::Up | KeyCode::Char('k') => {
                                        if state.selected > 0 {
                                            state.selected -= 1;
                                        }
                                        state.status_message = None;
                                    }
                                    KeyCode::Down | KeyCode::Char('j') => {
                                        if state.selected + 1 < state.entries.len() {
                                            state.selected += 1;
                                        }
                                        state.status_message = None;
                                    }
                                    KeyCode::Enter | KeyCode::Char(' ') => {
                                        // Toggle reveal for selected entry
                                        if state.revealed.contains(&state.selected) {
                                            state.revealed.remove(&state.selected);
                                        } else {
                                            state.revealed.insert(state.selected);
                                        }
                                    }
                                    KeyCode::Char('r') => {
                                        // Reveal all
                                        for i in 0..state.entries.len() {
                                            state.revealed.insert(i);
                                        }
                                    }
                                    KeyCode::Char('H') => {
                                        // Hide all (shifted to avoid conflict with vim left)
                                        state.revealed.clear();
                                    }
                                    KeyCode::Char('y') => {
                                        // Copy password to clipboard
                                        if !state.entries.is_empty() {
                                            if let Ok(mut clipboard) = Clipboard::new() {
                                                let pwd = &state.entries[state.selected].password;
                                                if clipboard.set_text(pwd.clone()).is_ok() {
                                                    state.status_message =
                                                        Some("✓ Copied to clipboard!".into());
                                                } else {
                                                    state.status_message =
                                                        Some("✗ Failed to copy".into());
                                                }
                                            } else {
                                                state.status_message =
                                                    Some("✗ Clipboard unavailable".into());
                                            }
                                        }
                                    }
                                    KeyCode::Char('d') => {
                                        // Confirm delete
                                        if !state.entries.is_empty() {
                                            *mode = ViewMode::ConfirmDelete;
                                        }
                                    }
                                    KeyCode::Char('e') => {
                                        // Start editing name
                                        if !state.entries.is_empty() {
                                            state.edit_buffer =
                                                state.entries[state.selected].name.clone();
                                            *mode = ViewMode::EditName;
                                        }
                                    }
                                    KeyCode::Char('p') => {
                                        // Start editing password
                                        if !state.entries.is_empty() {
                                            state.edit_buffer =
                                                state.entries[state.selected].password.clone();
                                            state.revealed.insert(state.selected);
                                            *mode = ViewMode::EditPassword;
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            ViewMode::ConfirmDelete => {
                                match key.code {
                                    KeyCode::Char('y') | KeyCode::Enter => {
                                        // Confirm delete
                                        if let Some(ref store) = storage {
                                            match store.delete(state.selected) {
                                                Ok(_) => {
                                                    state.entries.remove(state.selected);
                                                    if state.selected >= state.entries.len()
                                                        && state.selected > 0
                                                    {
                                                        state.selected -= 1;
                                                    }
                                                    state.revealed.clear();
                                                    state.status_message =
                                                        Some("✓ Deleted!".into());
                                                }
                                                Err(e) => {
                                                    state.status_message = Some(format!("✗ {}", e));
                                                }
                                            }
                                        }
                                        *mode = ViewMode::Browse;
                                    }
                                    KeyCode::Char('n') | KeyCode::Esc => {
                                        // Cancel delete
                                        *mode = ViewMode::Browse;
                                        state.status_message = None;
                                    }
                                    _ => {}
                                }
                            }
                            ViewMode::EditName => {
                                match key.code {
                                    KeyCode::Esc => {
                                        *mode = ViewMode::Browse;
                                        state.edit_buffer.clear();
                                        state.status_message = None;
                                    }
                                    KeyCode::Enter => {
                                        // Save name change
                                        if !state.edit_buffer.trim().is_empty() {
                                            if let Some(ref store) = storage {
                                                let mut entry =
                                                    state.entries[state.selected].clone();
                                                entry.name = state.edit_buffer.clone();
                                                match store.update(state.selected, entry.clone()) {
                                                    Ok(_) => {
                                                        state.entries[state.selected] = entry;
                                                        state.status_message =
                                                            Some("✓ Name updated!".into());
                                                    }
                                                    Err(e) => {
                                                        state.status_message =
                                                            Some(format!("✗ {}", e));
                                                    }
                                                }
                                            }
                                        }
                                        state.edit_buffer.clear();
                                        *mode = ViewMode::Browse;
                                    }
                                    KeyCode::Backspace => {
                                        state.edit_buffer.pop();
                                    }
                                    KeyCode::Char(c) => {
                                        state.edit_buffer.push(c);
                                    }
                                    _ => {}
                                }
                            }
                            ViewMode::EditPassword => {
                                match key.code {
                                    KeyCode::Esc => {
                                        *mode = ViewMode::Browse;
                                        state.edit_buffer.clear();
                                        state.status_message = None;
                                    }
                                    KeyCode::Enter => {
                                        // Save password change
                                        if !state.edit_buffer.is_empty() {
                                            if let Some(ref store) = storage {
                                                let mut entry =
                                                    state.entries[state.selected].clone();
                                                entry.password = state.edit_buffer.clone();
                                                match store.update(state.selected, entry.clone()) {
                                                    Ok(_) => {
                                                        state.entries[state.selected] = entry;
                                                        state.status_message =
                                                            Some("✓ Password updated!".into());
                                                    }
                                                    Err(e) => {
                                                        state.status_message =
                                                            Some(format!("✗ {}", e));
                                                    }
                                                }
                                            }
                                        }
                                        state.edit_buffer.clear();
                                        *mode = ViewMode::Browse;
                                    }
                                    KeyCode::Backspace => {
                                        state.edit_buffer.pop();
                                    }
                                    KeyCode::Char(c) => {
                                        state.edit_buffer.push(c);
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
