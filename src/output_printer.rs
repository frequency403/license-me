use chrono::Local;
use std::fmt::Display;
use indicatif::ProgressBar;
use crate::error_collector::*;

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
    pub fn error_msg<T>(&mut self, msg: T)
        where T: Display, {
        let formatted_message = format!("[{} - {}] {}",
                                        ansi_term::Style::new().bold().paint(Local::now().format(" %F | %T").to_string()),
                                        ansi_term::Color::Red.bold().paint("ERROR"),
                                        msg);
        eprintln!("{}", formatted_message);
        self.err_col.add(formatted_message);
    }
    pub fn normal_msg<T>(&self, msg: T)
        where T: Display, {
        println!("{}", msg);
    }
    pub fn verbose_msg<T>(&self, msg: T, bar_opt: Option<&ProgressBar>)
        where T: Display, {
        if self.verbose || self.debug {
            if let Some(bar) = bar_opt {
                bar.suspend(|| {
                    eprintln!(
                        "[{} - {}] {}",
                        ansi_term::Style::new().bold().paint(Local::now().format(" %F | %T").to_string()),
                        ansi_term::Color::Purple.bold().paint("INFO"),
                        msg
                    );
                })
            } else {
                eprintln!(
                    "[{} - {}] {}",
                    ansi_term::Style::new().bold().paint(Local::now().format(" %F | %T").to_string()),
                    ansi_term::Color::Purple.bold().paint("INFO"),
                    msg
                );
            }
        }
    }
    pub fn debug_msg<T>(&self, msg: T, bar_opt: Option<&ProgressBar>)
        where T: Display, {
        if self.debug {
            if let Some(bar) = bar_opt {
                bar.suspend(|| {
                    eprintln!(
                        "[{} - {}] {}",
                        ansi_term::Style::new().bold().paint(Local::now().format(" %F | %T").to_string()),
                        ansi_term::Color::Green.bold().paint("DEBUG"),
                        msg
                    );
                })
            } else {
                eprintln!(
                    "[{} - {}] {}",
                    ansi_term::Style::new().bold().paint(Local::now().format(" %F | %T").to_string()),
                    ansi_term::Color::Green.bold().paint("DEBUG"),
                    msg
                );
            }
        }
    }
}
