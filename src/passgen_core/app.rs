use rand::Rng;

/// Available input fields
#[derive(PartialEq, Clone, Copy)]
pub enum InputField {
    Name,
    Length,
    ToggleSpecial,
    ToggleLetters,
    ToggleNumbers,
    Generate,
}

impl InputField {
    /// Move to the next field
    pub fn next(self) -> Self {
        match self {
            Self::Name => Self::Length,
            Self::Length => Self::ToggleSpecial,
            Self::ToggleSpecial => Self::ToggleLetters,
            Self::ToggleLetters => Self::ToggleNumbers,
            Self::ToggleNumbers => Self::Generate,
            Self::Generate => Self::Name,
        }
    }

    /// Move to the previous field
    pub fn prev(self) -> Self {
        match self {
            Self::Name => Self::Generate,
            Self::Length => Self::Name,
            Self::ToggleSpecial => Self::Length,
            Self::ToggleLetters => Self::ToggleSpecial,
            Self::ToggleNumbers => Self::ToggleLetters,
            Self::Generate => Self::ToggleNumbers,
        }
    }
}

/// Main application state
pub struct App {
    pub name_input: String,
    pub length_input: String,
    pub use_special: bool,
    pub use_letters: bool,
    pub use_numbers: bool,
    pub active_field: InputField,
    pub generated_password: Option<String>,
    pub error: Option<String>,
    pub status_message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            name_input: String::new(),
            length_input: String::from("16"),
            use_special: true,
            use_letters: true,
            use_numbers: true,
            active_field: InputField::Name,
            generated_password: None,
            error: None,
            status_message: None,
        }
    }

    /// Generate a password based on current settings
    pub fn generate(&mut self) {
        self.error = None;
        self.status_message = None;
        self.generated_password = None;

        // Validate name
        if self.name_input.trim().is_empty() {
            self.error = Some("Please enter a password name".into());
            return;
        }

        // Validate length
        let length: usize = match self.length_input.parse() {
            Ok(n) if n > 0 && n <= 128 => n,
            Ok(_) => {
                self.error = Some("Length must be 1-128".into());
                return;
            }
            Err(_) => {
                self.error = Some("Invalid length".into());
                return;
            }
        };

        // Build character set
        let mut charset = String::new();

        if self.use_letters {
            charset.push_str("abcdefghijklmnopqrstuvwxyz");
            charset.push_str("ABCDEFGHIJKLMNOPQRSTUVWXYZ");
        }

        if self.use_numbers {
            charset.push_str("0123456789");
        }

        if self.use_special {
            charset.push_str("!@#$%^&*()_+-=[]{}|;:,.<>?");
        }

        if charset.is_empty() {
            self.error = Some("Enable at least one character type".into());
            return;
        }

        // Generate password
        let mut rng = rand::rng();
        let chars: Vec<char> = charset.chars().collect();
        let password: String = (0..length)
            .map(|_| chars[rng.random_range(0..chars.len())])
            .collect();

        self.generated_password = Some(password);
    }

    /// Toggle the current field if it's a toggle
    pub fn toggle_current(&mut self) {
        match self.active_field {
            InputField::ToggleSpecial => self.use_special = !self.use_special,
            InputField::ToggleLetters => self.use_letters = !self.use_letters,
            InputField::ToggleNumbers => self.use_numbers = !self.use_numbers,
            InputField::Generate => self.generate(),
            _ => {}
        }
    }

    /// Get the current text input field (if any)
    pub fn current_text_input(&mut self) -> Option<&mut String> {
        match self.active_field {
            InputField::Name => Some(&mut self.name_input),
            InputField::Length => Some(&mut self.length_input),
            _ => None,
        }
    }

    /// Navigate to next field
    pub fn next_field(&mut self) {
        self.active_field = self.active_field.next();
    }

    /// Navigate to previous field
    pub fn prev_field(&mut self) {
        self.active_field = self.active_field.prev();
    }

    /// Get the current password entry for saving
    pub fn get_entry(&self) -> Option<super::storage::PasswordEntry> {
        self.generated_password
            .as_ref()
            .map(|pwd| super::storage::PasswordEntry {
                name: self.name_input.clone(),
                password: pwd.clone(),
                created_at: chrono_timestamp(),
            })
    }

    /// Clear inputs after successful save
    pub fn clear_for_next(&mut self) {
        self.name_input.clear();
        self.generated_password = None;
        self.active_field = InputField::Name;
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple timestamp without external dependency
fn chrono_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", duration.as_secs())
}
