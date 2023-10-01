use crate::PrintMode;

// The ErrorCollector is a simple Container, that collects all error messages when they
// occur, then print all its contents to stderr when the program is about to end.

#[derive(Debug, Clone)]
pub struct ErrorCollector {
    collection: Vec<String>,
}

impl ErrorCollector {
    // Init a empty vector

    pub const fn init() -> Self {
        Self {
            collection: vec![]
        }
    }
    // Add a string to the collection
    pub fn add(&mut self, msg: String) {
        self.collection.push(msg)
    }

    // Iterate over every entry and print ending message or contents of the error-list
    pub fn list_errors(&self, processed_dirs: usize, pm: &PrintMode) {
        if self.collection.is_empty() {
            pm.normal_msg(format!("\n\nDone! Processed {} directories successfully!\n", processed_dirs))
        } else {
            eprintln!("{}", ansi_term::Color::Red.bold().blink().paint("!![ERROR(S) OCCURRED]!!\n\n"));
            self.collection.iter().for_each(|entry| {
                if let Some(position) = self.collection.iter().position(|pos| pos == entry) {
                    eprintln!(
                        "[{} [{}] ] {}",
                        ansi_term::Color::Red.bold().paint("ERROR LIST"),
                        ansi_term::Color::Red.bold().paint(position.to_string()),
                        entry
                    );
                }
            });
        }
    }
}