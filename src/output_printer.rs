use std::fmt::Display;

use chrono::Local;
use indicatif::ProgressBar;

use crate::error_collector::*;

// Using a print mode for Pretty-CLI print, with easy usage for the rest of the code
// Throughout all the code, you can use the PrintMode and determine wherever the message
// is "normal", "debug", for "verbose" output or an error

// The PrintMode Structure will decide on the given bool's if the message is printed or not.

#[derive(Debug, Clone)]
pub struct PrintMode {
    pub(crate) verbose: bool,
    pub(crate) debug: bool,
    pub(crate) err_col: ErrorCollector,
}

impl PrintMode {
    /// Initializes a `PrintMode` with default settings.
    ///
    /// # Example
    ///
    /// ```
    /// use crate::norm;
    ///
    /// let print_mode = norm();
    /// ```
    pub const fn norm() -> PrintMode {
        PrintMode {
            verbose: false,
            debug: false,
            err_col: ErrorCollector::init(),
        }
    }


    /// Prints an error message to stderr and adds it to the error collection.
    ///
    /// # Arguments
    ///
    /// * `msg` - The error message to display. This can be of any type that implements the `Display` trait.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ansi_term::Color;
    /// # use chrono::Local;
    /// # use std::fmt::Display;
    /// # use error_handling::ErrorCollection;
    /// # use error_handling::ErrorHandler;
    /// # fn main() {
    /// #    let mut error_handler = ErrorHandler::new(ErrorCollection::new());
    /// #    let msg = "Something went wrong";
    /// #    error_handler.error_msg(msg);
    /// # }
    /// ```
    pub fn error_msg<T>(&mut self, msg: T)
        where
            T: Display,
    {
        let formatted_message = format!(
            "[{} - {}] {}",
            // Time in bold
            ansi_term::Style::new()
                .bold()
                .paint(Local::now().format(" %F | %T").to_string()),
            // "ERROR" in Red
            ansi_term::Color::Red.bold().paint("ERROR"),
            // And the message itself
            msg
        );
        // Print to stderr at error occurrence
        eprintln!("{}", formatted_message);
        // Add error string to the Collection
        self.err_col.add(formatted_message);
    }

    /// Prints a message to the console.
    ///
    /// # Arguments
    ///
    /// * `msg` - The message to be printed. Must implement the `Display` trait.
    ///
    /// # Example
    ///
    /// ```
    /// use my_module::MyStruct;
    ///
    /// let my_object = MyStruct::new();
    /// my_object.normal_msg("Hello, World!");
    /// ```
    pub fn normal_msg<T>(&self, msg: T)
        where
            T: Display,
    {
        println!("{}", msg);
    }


    /// Prints a verbose message if the mode is verbose or debug. If a `ProgressBar` instance is provided,
    /// it will be suspended while printing the message.
    ///
    /// # Arguments
    ///
    /// * `msg` - The message to be printed. Must implement `std::fmt::Display`.
    /// * `bar_opt` - An optional `ProgressBar` instance. If provided, the progress bar will be suspended while
    ///               printing the message.
    ///
    /// # Example
    ///
    /// ```
    /// use ansi_term::Color;
    /// use chrono::Local;
    /// use indicatif::ProgressBar;
    ///
    /// let verbose = true;
    /// let debug = false;
    /// let msg = "This is a verbose message";
    /// let bar_opt = Some(&ProgressBar::new(10));
    ///
    /// verbose_msg(msg, bar_opt);
    /// ```
    pub fn verbose_msg<T>(&self, msg: T, bar_opt: Option<&ProgressBar>)
        where
            T: Display,
    {
        // If the mode is verbose or debug, verbose messages are printed
        if self.verbose || self.debug {
            // If there is a ProgressBar running, suspend it for printing the message
            if let Some(bar) = bar_opt {
                bar.suspend(|| {
                    eprintln!(
                        "[{} - {}] {}",
                        // Time string in bold style
                        ansi_term::Style::new()
                            .bold()
                            .paint(Local::now().format(" %F | %T").to_string()),
                        // The word "INFO" in purple color
                        ansi_term::Color::Purple.bold().paint("INFO"),
                        msg
                    );
                })
            }
            // Else just print the message.
            else {
                eprintln!(
                    "[{} - {}] {}",
                    ansi_term::Style::new()
                        .bold()
                        .paint(Local::now().format(" %F | %T").to_string()),
                    ansi_term::Color::Purple.bold().paint("INFO"),
                    msg
                );
            }
        }
    }

    /// Converts a boolean value into a colored string representation.
    ///
    /// # Examples
    ///
    /// ```
    /// use ansi_term::Color;
    ///
    /// let true_bool = true;
    /// let false_bool = false;
    ///
    /// let true_str = colored_bools(&true_bool);
    /// assert_eq!(true_str, Color::Green.bold().paint("true").to_string());
    ///
    /// let false_str = colored_bools(&false_bool);
    /// assert_eq!(false_str, Color::Red.bold().paint("false").to_string());
    /// ```
    ///
    /// # Arguments
    ///
    /// * `boolean` - A reference to a boolean value.
    ///
    /// # Returns
    ///
    /// A string representation of the boolean value with applied ANSI color codes.
    pub fn colored_bools(boolean: &bool) -> String {
        let string = if *boolean {
            ansi_term::Color::Green.bold().paint("true").to_string()
        } else {
            ansi_term::Color::Red.bold().paint("false").to_string()
        };
        string
    }

    /// Prints a debug message if the `debug` flag is set to true.
    ///
    /// # Arguments
    ///
    /// * `msg`: The message to print, must implement the `Display` trait.
    /// * `bar_opt`: An optional reference to a progress bar.
    ///
    /// # Example
    ///
    /// ```
    /// use ansi_term::Color;
    /// use chrono::Local;
    /// use progress_bar::ProgressBar;
    ///
    /// let debug = true;
    /// let pb = ProgressBar::new();
    /// let message = "This is a debug message.";
    ///
    /// debug_msg(message, Some(&pb));
    /// ```
    pub fn debug_msg<T>(&self, msg: T, bar_opt: Option<&ProgressBar>)
        where
            T: Display,
    {
        if self.debug {
            if let Some(bar) = bar_opt {
                bar.suspend(|| {
                    eprintln!(
                        "[{} - {}] {}",
                        // Time string in bold style
                        ansi_term::Style::new()
                            .bold()
                            .paint(Local::now().format(" %F | %T").to_string()),
                        // The word "DEBUG" in green color
                        ansi_term::Color::Green.bold().paint("DEBUG"),
                        msg
                    );
                })
            } else {
                eprintln!(
                    "[{} - {}] {}",
                    ansi_term::Style::new()
                        .bold()
                        .paint(Local::now().format(" %F | %T").to_string()),
                    ansi_term::Color::Green.bold().paint("DEBUG"),
                    msg
                );
            }
        }
    }
}
