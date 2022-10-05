use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;
use sysinfo::{DiskExt, System, SystemExt};
use walkdir::WalkDir;

use crate::{clear_term, PrintMode};


fn progress_spinner() -> ProgressBar {
    let p_bar = ProgressBar::new_spinner();
    p_bar.set_style(
        ProgressStyle::with_template("\n{msg}{spinner}").unwrap().tick_strings(&[".   ", " .  ", "  . ", "   .", "  . ", " .  ", " finished!"]),
    );
    p_bar.set_message("Searching");
    p_bar.enable_steady_tick(std::time::Duration::from_secs_f32(0.1));
    p_bar
}


fn walk(
    root: String,
    progress_bar: &ProgressBar,
    lrm: (bool, bool),
    pm: &PrintMode,
) -> Vec<String> {
    let vec: Vec<String> = WalkDir::new(root).into_iter().filter_map(|entry| {
        if let Ok(entry) = &entry {
            let mut s: String = String::new();
            pm.debug_msg_b(format!("Searching in: {}", &entry.path().display()), progress_bar);
            if entry.path().display().to_string().ends_with(".git") && !entry.path().display().to_string().contains('$') && !entry.path().display().to_string().contains(".cargo") {
                pm.verbose_msg_b(format!(
                    "Found: {}",
                    &entry.path().display().to_string().replace(".git", "")
                ), progress_bar);
                let walker: PathBuf = entry.path().display().to_string().replace(".git", "LICENSE").into();
                if (lrm.0 || lrm.1) && walker.exists() {
                    pm.verbose_msg_b("Found License file in directory", progress_bar);
                    pm.debug_msg_b("Adding found directory to collection...", progress_bar);
                    s = entry.path().display().to_string();
                } else if !(lrm.0 || lrm.1 || walker.exists()) {
                    pm.verbose_msg_b("Found no License file in directory", progress_bar);
                    pm.debug_msg_b("Adding found directory to collection...", progress_bar);
                    s = entry.path().display().to_string();
                }
                Some(s)
            } else {
                None
            }
        } else {
            None
        }
    }).collect();
    vec
}
pub fn print_git_dirs(lrm: (bool, bool), pm: &PrintMode) -> Vec<String> {
    clear_term();
    let bar = progress_spinner();
    pm.debug_msg_b("Initiation successful", &bar);
    let mut collector: Vec<String> = vec![];
    if cfg!(windows) {
        pm.verbose_msg_b("Windows Filesystem Mode", &bar);
        let sys = System::new_all();
        for disk in sys.disks() {
            pm.debug_msg_b(format!("Processing: {}", disk.mount_point().display()), &bar);
            walk(disk.mount_point().display().to_string(), &bar, lrm, pm).into_iter().filter_map(|item| {
                if !item.is_empty() {
                    pm.debug_msg_b(&item, &bar);
                    Some(item)
                } else {
                    None
                }
            }).for_each(|content| {
                pm.debug_msg_b(format!("Collected: {}", content), &bar);
                collector.push(content)
            });
        }
    } else {
        pm.verbose_msg_b("Unix Filesystem Mode", &bar);
        walk("/".to_string(), &bar, lrm, pm).into_iter().filter_map(|item| {
            if !item.is_empty() {
                pm.debug_msg_b(&item, &bar);
                Some(item)
            } else {
                None
            }
        }).for_each(|content| {
            pm.debug_msg_b(format!("Collected: {}", content), &bar);
            collector.push(content)
        });
    }
    bar.finish();
    clear_term();
    if collector.is_empty() {
        pm.normal_msg("Found no possible unlicensed git repository's! Exiting....");
        std::process::exit(1);
    }
    if lrm.0 || lrm.1 {
        pm.normal_msg(format!(
            "Found {} possible repository(s) that have a LICENSE!\n\n",
            collector.len()
        ));
    } else {
        pm.normal_msg(format!(
            "Found {} possible repository(s) that have no LICENSE!\n\n",
            collector.len()
        ));
    }
    collector.iter().for_each(|i| {
        if let Some(int) = collector.iter().position(|l| l == i) {
            //@TODO failsafe for dirs containing ".git"
            pm.normal_msg(format!("[{}] \"{}\"", int + 1, i.replace(".git", "")));
        }
    });

    collector
}

