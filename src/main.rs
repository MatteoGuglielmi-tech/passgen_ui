use std::io;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use passgen_ui::passgen_core::{app::App, storage::Storage, ui};

/// Application phase
enum Phase {
    MasterPassword,
    Main,
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

    loop {
        // Render
        terminal.draw(|f| {
            let show_master = matches!(phase, Phase::MasterPassword);
            ui::render(f, &app, show_master, &master_input);
        })?;

        // Handle input
        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }

            match phase {
                Phase::MasterPassword => {
                    match key.code {
                        KeyCode::Esc => return Ok(()),
                        KeyCode::Enter => {
                            if master_input.is_empty() {
                                continue;
                            }
                            match Storage::new(&master_input) {
                                Ok(s) => {
                                    storage = Some(s);
                                    phase = Phase::Main;
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
                    }
                }
                Phase::Main => {
                    match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Esc => return Ok(()),
                        KeyCode::Tab | KeyCode::Down => app.next_field(),
                        KeyCode::BackTab | KeyCode::Up => app.prev_field(),
                        KeyCode::Enter => {
                            app.generate();
                            // Auto-save if generation succeeded
                            if app.generated_password.is_some() {
                                if let Some(ref store) = storage {
                                    if let Some(entry) = app.get_entry() {
                                        match store.save(entry) {
                                            Ok(_) => {
                                                app.status_message = Some(format!(
                                                    "✓ Saved to {}",
                                                    store.path().display()
                                                ));
                                            }
                                            Err(e) => {
                                                app.error = Some(format!("Save failed: {}", e));
                                            }
                                        }
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
            }
        }
    }
}
