use std::fmt::Display;
use std::path::MAIN_SEPARATOR;
use futures::executor::block_on;

use sysinfo::{DiskExt, System, SystemExt};
use tokio::task::JoinHandle;
use tokio::time::Instant;
use walkdir::WalkDir;

use crate::git_dir::GitDir;
use crate::github_license::GithubLicense;
use crate::operating_mode::OperatingMode;

pub async fn init_search(
    op_mode: OperatingMode,
    time: Instant,
    licenses: Vec<GithubLicense>,
) -> Vec<GitDir> {
    let system = System::new_all();
    let mut task_holder: Vec<JoinHandle<Vec<GitDir>>> = vec![];
    system.disks().iter().for_each(|disk| {
        task_holder.push(tokio::spawn(start_walking(
            disk.mount_point().display().to_string(),
            op_mode,
            licenses.clone(),
        )))
    });
    let mut dirs: Vec<GitDir> = vec![];
    futures::future::join_all(task_holder)
        .await
        .iter()
        .filter_map(|dir| {
            if let Ok(result) = dir {
                Some(result)
            } else {
                None
            }
        })
        .for_each(|dir| {
            dir.iter().for_each(|git_dir| {
                dirs.push(git_dir.clone());
            });
        });
    println!("Searching took: {}s", time.elapsed().as_secs());
    dirs
}

async fn start_walking<T>(
    root: T,
    op_mode: OperatingMode,
    licences: Vec<GithubLicense>,
) -> Vec<GitDir>
where
    T: Display,
{
    let mut task_holder: Vec<JoinHandle<Vec<GitDir>>> = vec![];
    WalkDir::new(root.to_string())
        .max_depth(1)
        .into_iter()
        .for_each(|dir| {
            if let Ok(entry) = dir {
                let tmp = entry.path().display().to_string();
                if !tmp.contains('$') {
                    task_holder.push(tokio::spawn(walk_deeper(tmp, op_mode, licences.clone())))
                }
            }
        });
    let mut any_dir: Vec<GitDir> = vec![];
    futures::future::join_all(task_holder)
        .await
        .iter()
        .filter_map(|future| if let Ok(f) = future { Some(f) } else { None })
        .for_each(|res| {
            res.to_vec().iter().for_each(|item| {
                if !any_dir.contains(item) {
                    any_dir.push(item.clone())
                }
            })
        });
    any_dir
}

async fn walk_deeper(
    root: String,
    op_mode: OperatingMode,
    licenses: Vec<GithubLicense>,
) -> Vec<GitDir> {
    WalkDir::new(root)
        .into_iter()
        .filter_map(|p_dir| {
            if let Ok(valid_dir) = p_dir {
                let path = valid_dir.path().display().to_string();
                if path.ends_with(format!("{}{}", MAIN_SEPARATOR, ".git").as_str())
                    && !path.contains(".cargo")
                    && !path.contains('$')
                    && !path.contains("AppData")
                {
                    let dir = block_on(GitDir::init(path, Some(licenses.clone())));
                    match op_mode {
                        OperatingMode::SetNewLicense => {
                            if dir.license_path.is_none() {
                                Some(dir)
                            } else {
                                None
                            }
                        }
                        OperatingMode::AppendLicense => {
                            if dir.license_path.is_some() {
                                Some(dir)
                            } else {
                                None
                            }
                        }
                        OperatingMode::LicenseReplace => {
                            if dir.license_path.is_some() {
                                Some(dir)
                            } else {
                                None
                            }
                        }
                        _ => Some(dir),
                    }
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect()
}
