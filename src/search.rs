use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;
use std::time::Instant;
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
    lrm: (bool, bool, bool),
    pm: &mut PrintMode,
) -> Vec<String> {
    WalkDir::new(root).into_iter().filter_map(|entry| {
        if let Ok(entry) = &entry {
            pm.debug_msg(format!("Searching in: {}", &entry.path().display()), Some(progress_bar));
            if entry.path().display().to_string().ends_with(".git") && !entry.path().display().to_string().contains('$') && !entry.path().display().to_string().contains(".cargo") {
                pm.verbose_msg(format!(
                    "Found: {}",
                    &entry.path().display().to_string().replace(".git", "")
                ), Some(progress_bar));
                let walker: PathBuf = entry.path().display().to_string().replace(".git", "LICENSE").into();
                if lrm.2 {
                    pm.verbose_msg("Adding dir to collection..", Some(progress_bar));
                    Some(entry.path().display().to_string())
                } else if (lrm.0 || lrm.1) && walker.exists() {
                    pm.verbose_msg("Found License file in directory", Some(progress_bar));
                    pm.debug_msg("Adding found directory to collection...", Some(progress_bar));
                    Some(entry.path().display().to_string())
                } else if !(lrm.0 || lrm.1 || walker.exists()) {
                    pm.verbose_msg("Found no License file in directory", Some(progress_bar));
                    pm.debug_msg("Adding found directory to collection...", Some(progress_bar));
                    Some(entry.path().display().to_string())
                } else { None }
            } else {
                None
            }
        } else {
            None
        }
    }).collect::<Vec<String>>()
}

fn dir_validator(dir: String, collection: &mut Vec<String>, pm: &mut PrintMode, bar: &ProgressBar) {
    if let Some(buf) = dir.strip_suffix(".git") {
        if !buf.contains(".git") {
            pm.debug_msg(format!("Collected: {}", dir), Some(bar));
            pm.verbose_msg(format!("Valid: {}", dir), Some(bar));
            collection.push(dir)
        } else {
            pm.error_msg(format!("Invalid Directory: {}", dir))
        }
    }
}

pub fn print_git_dirs(lrm: (bool, bool, bool), pm: &mut PrintMode, dur: Instant) -> Vec<String> {
    clear_term();
    let bar = progress_spinner();
    let mut prm = pm.clone();
    pm.debug_msg("Initiation successful", Some(&bar));
    let mut collector: Vec<String> = vec![];
    if cfg!(windows) {
        pm.verbose_msg("Windows Filesystem Mode", Some(&bar));
        let sys = System::new_all();
        for disk in sys.disks() {
            pm.debug_msg(format!("Processing: {}", disk.mount_point().display()), Some(&bar));
            walk(disk.mount_point().display().to_string(), &bar, lrm, pm).into_iter().filter_map(|item| {
                if !item.is_empty() {
                    pm.debug_msg(&item, Some(&bar));
                    Some(item)
                } else {
                    None
                }
            }).for_each(|content| {
                dir_validator(content, &mut collector, &mut prm, &bar)
            });
        }
    } else {
        pm.verbose_msg("Unix Filesystem Mode", Some(&bar));
        walk("/".to_string(), &bar, lrm, pm).into_iter().filter_map(|item| {
            if !item.is_empty() {
                pm.debug_msg(&item, Some(&bar));
                Some(item)
            } else {
                None
            }
        }).for_each(|content| {
            dir_validator(content, &mut collector, &mut prm, &bar)
        });
    }
    bar.finish();
    clear_term();
    if collector.is_empty() {
        pm.normal_msg("Found no possible unlicensed git repository's! Exiting....");
        std::process::exit(1);
    }
    if lrm.2 {
        pm.normal_msg(format!(
            "Found {} possible repository(s)! Took {:.2} secs!\n\n",
            collector.len(), dur.elapsed().as_secs_f32()
        ));
    } else if lrm.0 || lrm.1 {
        pm.normal_msg(format!(
            "Found {} possible repository(s) that have a LICENSE! Took {:.2} secs!\n\n",
            collector.len(), dur.elapsed().as_secs_f32()
        ));
    } else {
        pm.normal_msg(format!(
            "Found {} possible repository(s) that have no LICENSE! Took {:.2} secs!\n\n",
            collector.len(), dur.elapsed().as_secs_f32()
        ));
    }
    collector.iter().for_each(|i| {
        if let Some(int) = collector.iter().position(|l| l == i) {
            pm.normal_msg(format!("[{}] \"{}\"", int + 1, i.replace(".git", "")));
        }
    });
    collector
}

