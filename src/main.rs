use std::io;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

use passgen_ui::passgen_core::{app::App, storage::Storage, ui};

/// Application phase
enum Phase {
    MasterPassword,
    Main,
    ChangeMasterPassword { step: ChangeStep },
}

enum ChangeStep {
    EnterOld,
    EnterNew,
    ConfirmNew,
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
            }
        }
    }
}
