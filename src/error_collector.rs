use crate::PrintMode;

// The ErrorCollector is a simple Container, that collects all error messages when they
// occur, then print all its contents to stderr when the program is about to end.

/// A collection of errors.
#[derive(Debug, Clone)]
pub struct ErrorCollector {
    collection: Vec<String>,
}

impl ErrorCollector {
    /// Initializes a new instance of `Collection` with an empty collection.
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::Collection;
    ///
    /// let collection = Collection::init();
    /// assert!(collection.is_empty());
    /// ```
    pub const fn init() -> Self {
        Self { collection: vec![] }
    }


    /// Adds a message to the collection.
    ///
    /// # Arguments
    ///
    /// * `msg` - A `String` representing the message to add.
    ///
    /// # Example
    ///
    /// ```
    /// let mut collection = Collection::new();
    /// collection.add("Hello, world!".to_string());
    /// ```
    pub fn add(&mut self, msg: String) {
        self.collection.push(msg)
    }


    /// This function is used to list errors.
    ///
    /// # Arguments
    ///
    /// - `self`: a reference to the current instance of the struct.
    /// - `processed_dirs`: the number of directories processed successfully.
    /// - `pm`: a reference to the `PrintMode` for displaying messages.
    ///
    /// # Example
    ///
    /// ```
    /// use ansi_term::Colour;
    ///
    /// // create a struct instance
    /// let instance = YourStruct::new();
    ///
    /// // call the list_errors function
    /// instance.list_errors(10, &PrintMode::default());
    /// ```
    pub fn list_errors(&self, processed_dirs: usize, pm: &PrintMode) {
        if self.collection.is_empty() {
            pm.normal_msg(format!(
                "\n\nDone! Processed {} directories successfully!\n",
                processed_dirs
            ))
        } else {
            eprintln!(
                "{}",
                ansi_term::Color::Red
                    .bold()
                    .blink()
                    .paint("!![ERROR(S) OCCURRED]!!\n\n")
            );
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
