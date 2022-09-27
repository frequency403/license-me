use std::path::PathBuf;
use indicatif::{ProgressBar, ProgressStyle};
use log::info;
use walkdir::WalkDir;
use crate::clear_term;
use sysinfo::{DiskExt, System, SystemExt};

pub fn print_git_dirs() -> Vec<String> {
    clear_term();
    let bar = ProgressBar::new_spinner();
    bar.set_style(
        ProgressStyle::with_template("\n{msg}{spinner}")
            .unwrap()
            .tick_strings(&[
                ".  ",
                " . ",
                "  .",
                " . ",
                ".  ",
                "   ",
                " finished!"
            ]),
    );
    bar.set_message("Searching");
    let mut collector: Vec<String> = vec![];
    if std::env::consts::FAMILY.contains("win") {
        info!("Windows Filesystem Mode");
        let sys = System::new_all();
        for disk in sys.disks() {
            let vec: Vec<String> = WalkDir::new(disk.mount_point().display().to_string()).into_iter().filter_map(|entry| {
                if let Ok(entry) = &entry {
                    let mut s: String = String::new();
                    bar.tick();
                    if entry.path().display().to_string().ends_with(".git") &&
                        !entry.path().display().to_string().contains('$') &&
                        !entry.path().display().to_string().contains(".cargo") {
                        let walker: PathBuf = entry.path().display().to_string().replace(".git", "LICENSE").into();
                        if !walker.exists() {
                            s = entry.path().display().to_string();
                        }
                        Some(s)
                    } else { None }
                } else { None }
            }).collect();
            vec.into_iter().filter_map(|item| {
                if !item.is_empty() {
                    Some(item)
                } else { None }
            }).for_each(|content| {
                collector.push(content)
            });
        }
    } else {
        info!("Unix Filesystem Mode");
        let vec: Vec<String> = WalkDir::new("/").into_iter().filter_map(|entry| {
            if let Ok(entry) = &entry {
                let mut s: String = String::new();
                bar.tick();
                if entry.path().display().to_string().ends_with(".git") &&
                    !entry.path().display().to_string().contains('$') &&
                    !entry.path().display().to_string().contains(".cargo") {
                    let walker: PathBuf = entry.path().display().to_string().replace(".git", "LICENSE").into();
                    if !walker.exists() {
                        s = entry.path().display().to_string();
                    }
                    Some(s)
                } else { None }
            } else { None }
        }).collect();
        vec.into_iter().filter_map(|item| {
            if !item.is_empty() {
                Some(item)
            } else { None }
        }).for_each(|content| {
            collector.push(content)
        });
    }

    bar.finish();
    clear_term();
    if collector.is_empty() {
        println!("Found no possible unlicensed git repository's! Exiting....");
        std::process::exit(1);
    }
    println!("Found {} possible repository(s) that have no LICENSE!\n\n", collector.len());
    collector.iter().for_each(|i| {
        if let Some(int) = collector.iter().position(|l| l == i) {
            println!("[{}] \"{}\"", int + 1, i.replace(".git", ""));
        }
    });

    collector
}