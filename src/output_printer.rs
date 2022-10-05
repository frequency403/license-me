use chrono::Local;
use std::fmt::Display;
use indicatif::ProgressBar;

#[derive(Debug, Copy, Clone)]
pub struct PrintMode {
    pub(crate) verbose: bool,
    pub(crate) debug: bool,
}

impl PrintMode {
    pub const fn norm() -> PrintMode {
        PrintMode {
            verbose: false,
            debug: false,
        }
    }
    pub fn verbose_msg<T>(&self, msg: T)
    where
        T: Display,
    {
        if self.verbose || self.debug {
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
    pub fn debug_msg<T>(&self, msg: T)
    where
        T: Display,
    {
        if self.debug {
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
    pub fn verbose_msg_b<T>(&self, msg: T, bar: &ProgressBar)
        where
            T: Display,
    {
        if self.verbose || self.debug {
            bar.suspend(|| {
                eprintln!(
                    "[{} - {}] {}",
                    ansi_term::Style::new()
                        .bold()
                        .paint(Local::now().format(" %F | %T").to_string()),
                    ansi_term::Color::Purple.bold().paint("INFO"),
                    msg
                );
            })
        }
    }
    pub fn debug_msg_b<T>(&self, msg: T, bar: &ProgressBar)
        where
            T: Display,
    {
        if self.debug {
            bar.suspend(|| {
                eprintln!(
                    "[{} - {}] {}",
                    ansi_term::Style::new()
                        .bold()
                        .paint(Local::now().format(" %F | %T").to_string()),
                    ansi_term::Color::Green.bold().paint("DEBUG"),
                    msg
                );
            })
        }
    }
    //@TODO impl error collection
    pub fn error_msg<T>(&self, msg: T)
    where
        T: Display,
    {
        eprintln!(
            "[{} - {}] {}",
            ansi_term::Style::new()
                .bold()
                .paint(Local::now().format(" %F | %T").to_string()),
            ansi_term::Color::Red.bold().paint("ERROR"),
            msg
        );
    }
    pub fn normal_msg<T>(&self, msg: T)
    where
        T: Display,
    {
        println!("{}", msg);
    }
}
