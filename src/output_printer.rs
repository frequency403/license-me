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
    pub const fn norm() -> PrintMode {
        PrintMode {
            verbose: false,
            debug: false,
            err_col: ErrorCollector::init(),
        }
    }

    // In all functions, "msg" is Generic, which implements the "Display" trait.
    // So you can use "&str" and "String" as Parameters

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

    // A normal message printed to stdout
    pub fn normal_msg<T>(&self, msg: T)
    where
        T: Display,
    {
        println!("{}", msg);
    }

    // Messages for the Verbose mode with time
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

    pub fn colored_bools(boolean: bool) -> String {
        let string = if boolean {
            ansi_term::Color::Green.bold().paint("true").to_string()
        } else {
            ansi_term::Color::Red.bold().paint("false").to_string()
        };
        string
    }
    // Same applies for the debug messages
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
