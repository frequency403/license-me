use crate::clear_term;
use indicatif::{ProgressBar, ProgressStyle};
use log::{debug, info};
use std::path::PathBuf;
use sysinfo::{DiskExt, System, SystemExt};
use walkdir::WalkDir;

fn progress_spinner() -> ProgressBar {
    let p_bar = ProgressBar::new_spinner();
    p_bar.set_style(
        ProgressStyle::with_template("\n{msg}{spinner}")
            .unwrap()
            .tick_strings(&[".   ", " .  ", "  . ", "   .", "  . ", " .  ", " finished!"]),
    );
    p_bar.set_message("Searching");
    p_bar
}

fn walk(root: String, progress_bar: &ProgressBar, lrm: (bool, bool)) -> Vec<String> {
    let vec: Vec<String> = WalkDir::new(root)
        .into_iter()
        .filter_map(|entry| {
            if let Ok(entry) = &entry {
                let mut s: String = String::new();
                progress_bar.tick();
                debug!("Searching in: {}", &entry.path().display());
                if entry.path().display().to_string().ends_with(".git")
                    && !entry.path().display().to_string().contains('$')
                    && !entry.path().display().to_string().contains(".cargo")
                {
                    info!(
                        "Found: {}",
                        &entry.path().display().to_string().replace(".git", "")
                    );
                    let walker: PathBuf = entry
                        .path()
                        .display()
                        .to_string()
                        .replace(".git", "LICENSE")
                        .into();
                    if !walker.exists() || lrm.0 || lrm.1 {
                        debug!("Adding found directory to collection...");
                        s = entry.path().display().to_string();
                    }
                    Some(s)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();
    vec
}

pub fn print_git_dirs(lrm: (bool, bool)) -> Vec<String> {
    clear_term();
    let bar = progress_spinner();
    debug!("Initiation successful");
    let mut collector: Vec<String> = vec![];
    if std::env::consts::FAMILY.contains("win") {
        info!("Windows Filesystem Mode");
        let sys = System::new_all();
        for disk in sys.disks() {
            walk(disk.mount_point().display().to_string(), &bar, lrm)
                .into_iter()
                .filter_map(|item| if !item.is_empty() { Some(item) } else { None })
                .for_each(|content| collector.push(content));
        }
    } else {
        info!("Unix Filesystem Mode");
        walk("/".to_string(), &bar, lrm)
            .into_iter()
            .filter_map(|item| if !item.is_empty() { Some(item) } else { None })
            .for_each(|content| collector.push(content));
    }

    bar.finish();
    clear_term();
    if collector.is_empty() {
        println!("Found no possible unlicensed git repository's! Exiting....");
        std::process::exit(1);
    }
    println!(
        "Found {} possible repository(s) that have no LICENSE!\n\n",
        collector.len()
    );
    collector.iter().for_each(|i| {
        if let Some(int) = collector.iter().position(|l| l == i) {
            println!("[{}] \"{}\"", int + 1, i.replace(".git", ""));
        }
    });

    collector
}
