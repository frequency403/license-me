use indicatif::{ProgressBar, ProgressStyle};
use std::time::Instant;
use sysinfo::{DiskExt, System, SystemExt};
use tokio::task::JoinHandle;
use walkdir::WalkDir;

use crate::{clear_term, PrintMode};
use crate::operating_mode::OperatingMode;

// Using a ProgressBar (spinner) from the crate "ProgressBar"
pub fn progress_spinner() -> ProgressBar {
    // Init main struct
    let p_bar = ProgressBar::new_spinner();
    // Set the style and the tick strings, iterating over all but not the last item every tick
    p_bar.set_style(
        ProgressStyle::with_template("\n{msg}{spinner}").unwrap().tick_strings(&[".   ", " .  ", "  . ", "   .", "  . ", " .  ", " finished!"]),
    );
    // The message shown at {msg}, must be set AFTER declaring the style
    p_bar.set_message("Searching");
    // Using steady tick for eye-friendliness
    p_bar.enable_steady_tick(std::time::Duration::from_secs_f32(0.1));
    // return the bar "object"
    p_bar
}

// This function "walks" through every directory and checks after a pattern, if the directory can be a git directory
// Then it returns the collection of dirs it found
async fn walk(
    root: String, // Can be "/" in Unix, or "C:\", "D:\", etc...
    progress_bar: &ProgressBar,
    lrm: &OperatingMode,
    pm: &mut PrintMode,
) -> Vec<String> {
    // Using Iterators for performance reasons
    // With the WalkDir crate, you can iterate over every directory on your computer
    WalkDir::new(root).into_iter().filter_map(|entry| {
        if let Ok(entry) = &entry {
            pm.debug_msg(format!("Searching in: {}", &entry.path().display()), Some(progress_bar));
            // Filter every DirEntry (any directory path) for a ".git" ending.
            // When the DirEntry contains a "$" or ".cargo", ignore the entry

            if entry.path().display().to_string().ends_with(".git") && !entry.path().display().to_string().contains('$') && !entry.path().display().to_string().contains(".cargo") {
                pm.verbose_msg(format!(
                    "Found: {}",
                    &entry.path().display().to_string().replace(".git", "")
                ), Some(progress_bar));

                // TODO Delete
                // let walker: PathBuf = entry.path().display().to_string().replace(".git", "LICENSE").into(); // Build a temporary License Path

                return match lrm {
                    OperatingMode::AppendLicense => {
                        // Add all dirs where a license file is found (Append or Replace mode)
                        pm.verbose_msg("Found License file in directory", Some(progress_bar));
                        pm.debug_msg("Adding found directory to collection...", Some(progress_bar));
                        Some(entry.path().display().to_string())
                    }
                    OperatingMode::LicenseReplace => {
                        // Add all dirs where a license file is found (Append or Replace mode)
                        pm.verbose_msg("Found License file in directory", Some(progress_bar));
                        pm.debug_msg("Adding found directory to collection...", Some(progress_bar));
                        Some(entry.path().display().to_string())
                    }
                    OperatingMode::ShowAllGitDirs => {
                        // Add all found dirs to the collection
                        pm.verbose_msg("Adding dir to collection..", Some(progress_bar));
                        Some(entry.path().display().to_string())
                    }
                    OperatingMode::SetNewLicense => {
                        // Add all dirs where no license file is found
                        pm.verbose_msg("Found no License file in directory", Some(progress_bar));
                        pm.debug_msg("Adding found directory to collection...", Some(progress_bar));
                        Some(entry.path().display().to_string())
                    }
                }
            } else {
                None
            }
        } else {
            None
        }
        // Collect results and return them
    }).collect::<Vec<String>>()
}

// Checks on disk, if given string (dir) contains a ".git" directory
// if it does, add it to borrowed collection outside of the function
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

// Prints the Git directories, containing the calls to the collector-functions
// The "dur" parameter is for time measuring

pub async fn print_git_dirs(lrm: &OperatingMode, pm: &mut PrintMode, dur: Instant) -> Vec<String> {
    clear_term();
    // Init Progress spinner
    let bar = progress_spinner();
    // Clone PrintMode for ownership reasons
    let mut prm = pm.clone();
    pm.debug_msg("Initiation successful", Some(&bar));

    // Placeholder
    let mut collector: Vec<String> = vec![];

    // If system is Windows....
    if cfg!(windows) {
        pm.verbose_msg("Windows Filesystem Mode", Some(&bar));
        // Use new crate "sysinfo", where any plugged disks can be shown
        let sys = System::new_all();

        for disk in sys.disks() {
            pm.debug_msg(format!("Processing: {}", disk.mount_point().display()), Some(&bar));
            // Every found disk is used as "root" variable for the "walk" function
            walk(disk.mount_point().display().to_string(), &bar, lrm, pm).await.into_iter().filter_map(|item| {
                if !item.is_empty() {
                    pm.debug_msg(&item, Some(&bar));
                    Some(item)
                } else {
                    None
                }
            }).for_each(|content| {
                // Validate each entry, adding results to the "collector" placeholder
                dir_validator(content, &mut collector, &mut prm, &bar)
            });
        }
    } else {
        pm.verbose_msg("Unix Filesystem Mode", Some(&bar));
        // Using "/" as the root for the "walk" function in Unix mode
        walk("/".to_string(), &bar, lrm, pm).await.into_iter().filter_map(|item| {
            if !item.is_empty() {
                pm.debug_msg(&item, Some(&bar));
                Some(item)
            } else {
                None
            }
        }).for_each(|content| {
            // Validate each entry, adding results to the "collector" placeholder
            dir_validator(content, &mut collector, &mut prm, &bar)
        });
    }
    // Finish the progress bar
    bar.finish();
    clear_term();

    // If nothing is found, exit program with message
    if collector.is_empty() {
        pm.normal_msg("Found no possible unlicensed git repository's! Exiting....");
        std::process::exit(1);
    }

    // Any of the following messages contains the elapsed time since invocation of the program.
    match lrm {
        OperatingMode::AppendLicense => {
            // If there is a license file in directory
            pm.normal_msg(format!(
                "Found {} possible repository(s) that have a LICENSE! Took {:.2} secs!\n\n",
                collector.len(), dur.elapsed().as_secs_f32()
            ));
            }
        OperatingMode::LicenseReplace => {
            // If there is a license file in directory
            pm.normal_msg(format!(
                "Found {} possible repository(s) that have a LICENSE! Took {:.2} secs!\n\n",
                collector.len(), dur.elapsed().as_secs_f32()
            ));}
        OperatingMode::ShowAllGitDirs => {
            // If all git dirs should be printed
            pm.normal_msg(format!(
                "Found {} possible repository(s)! Took {:.2} secs!\n\n",
                collector.len(), dur.elapsed().as_secs_f32()
            ));}
        OperatingMode::SetNewLicense => {
            // If there is no license file in directory
            pm.normal_msg(format!(
                "Found {} possible repository(s) that have no LICENSE! Took {:.2} secs!\n\n",
                collector.len(), dur.elapsed().as_secs_f32()
            ));
        }
    }

    // For each collector Entry, print out the directories
    // i.e. [1] C:\.....
    //      [2] D:\.....


    collector.iter().for_each(|i| {
        if let Some(int) = collector.iter().position(|l| l == i) {
            pm.normal_msg(format!("[{}] \"{}\"", int + 1, i.replace(".git", "")));
        }
    });
    // return the collection
    collector
}

