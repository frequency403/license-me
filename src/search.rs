use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;
use sysinfo::{DiskExt, System, SystemExt};
use walkdir::WalkDir;

use crate::{clear_term, PrintMode};

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

fn walk(
    root: String,
    progress_bar: &ProgressBar,
    lrm: (bool, bool),
    pm: &PrintMode,
) -> Vec<String> {
    let vec: Vec<String> = WalkDir::new(root)
        .into_iter()
        .filter_map(|entry| {
            if let Ok(entry) = &entry {
                let mut s: String = String::new();
                progress_bar.tick();
                pm.debug_msg(format!("Searching in: {}", &entry.path().display()));
                if entry.path().display().to_string().ends_with(".git")
                    && !entry.path().display().to_string().contains('$')
                    && !entry.path().display().to_string().contains(".cargo")
                {
                    pm.verbose_msg(format!(
                        "Found: {}",
                        &entry.path().display().to_string().replace(".git", "")
                    ));
                    let walker: PathBuf = entry
                        .path()
                        .display()
                        .to_string()
                        .replace(".git", "LICENSE")
                        .into();
                    if !walker.exists() || lrm.0 || lrm.1 {
                        pm.debug_msg("Adding found directory to collection...");
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

pub fn print_git_dirs(lrm: (bool, bool), pm: &PrintMode) -> Vec<String> {
    clear_term();
    let bar = progress_spinner();
    pm.debug_msg("Initiation successful");
    let mut collector: Vec<String> = vec![];
    if std::env::consts::FAMILY.contains("win") {
        pm.verbose_msg("Windows Filesystem Mode");
        let sys = System::new_all();
        for disk in sys.disks() {
            pm.debug_msg(format!("Processing: {}", disk.mount_point().display()));
            walk(disk.mount_point().display().to_string(), &bar, lrm, pm)
                .into_iter()
                .filter_map(|item| {
                    if !item.is_empty() {
                        pm.debug_msg(&item);
                        Some(item)
                    } else {
                        None
                    }
                })
                .for_each(|content| {
                    pm.debug_msg(format!("Collected: {}", content));
                    collector.push(content)
                });
        }
    } else {
        pm.verbose_msg("Windows Filesystem Mode");
        walk("/".to_string(), &bar, lrm, pm)
            .into_iter()
            .filter_map(|item| {
                if !item.is_empty() {
                    pm.debug_msg(&item);
                    Some(item)
                } else {
                    None
                }
            })
            .for_each(|content| {
                pm.debug_msg(format!("Collected: {}", content));
                collector.push(content)
            });
    }

    bar.finish();
    clear_term();
    if collector.is_empty() {
        pm.normal_msg("Found no possible unlicensed git repository's! Exiting....");
        std::process::exit(1);
    }
    pm.normal_msg(format!(
        "Found {} possible repository(s) that have no LICENSE!\n\n",
        collector.len()
    ));
    collector.iter().for_each(|i| {
        if let Some(int) = collector.iter().position(|l| l == i) {
            pm.normal_msg(format!("[{}] \"{}\"", int + 1, i.replace(".git", "")));
        }
    });

    collector
}
