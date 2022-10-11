use crate::PrintMode;

#[derive(Debug, Clone)]
pub struct ErrorCollector {
    collection: Vec<String>,
}

impl ErrorCollector {
    pub const fn init() -> Self {
        Self {
            collection: vec![]
        }
    }
    pub fn add(&mut self, msg: String) {
        self.collection.push(msg)
    }
    pub fn list_errors(&self, processed_dirs: usize, pm: &PrintMode) {
        if self.collection.is_empty() {
            pm.normal_msg(format!("\n\n Done! Processed {} directories successfully!\n", processed_dirs))
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