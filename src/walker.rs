use std::fmt::Display;
use std::path::MAIN_SEPARATOR;
use std::sync::mpsc::Sender;
use tokio::task::JoinHandle;
use walkdir::WalkDir;
use crate::git_dir::GitDir;
use crate::operating_mode::OperatingMode;


async fn walk_deeper(sender: Sender<usize>, root: String, op_mode: OperatingMode) -> Vec<GitDir> {
    WalkDir::new(root).into_iter().filter_map( |p_dir| {
        sender.send(1).unwrap();
        if let Ok(valid_dir) = p_dir {
            let path = valid_dir.path().display().to_string();
            if path.ends_with(format!("{}{}", MAIN_SEPARATOR, ".git").as_str()) &&
                !path.contains(".cargo") && !path.contains('$') &&
                !path.contains("AppData") {

                let dir = GitDir::init(path);
                match op_mode {
                    OperatingMode::SetNewLicense => {
                        if !dir.has_alicense {
                            Some(dir)
                        } else { None }
                    }
                    OperatingMode::AppendLicense => {
                        if dir.has_alicense {
                            Some(dir)
                        } else { None }
                    }
                    OperatingMode::LicenseReplace => {
                        if dir.has_alicense {
                            Some(dir)
                        } else { None }
                    }
                    _ => { Some(dir) }
                }
            } else {
                None
            }
        } else {
            None
        }
    }).collect()
}


pub async fn start_walking<T>(sender: Sender<usize>, root: T, op_mode: OperatingMode) -> Vec<GitDir> where T:Display {
    let mut task_holder: Vec<JoinHandle<Vec<GitDir>>> = vec![];
    WalkDir::new(root.to_string()).max_depth(1).into_iter().for_each(|dir| {
        sender.send(1).unwrap();
        if let Ok(entry) = dir {
            let tmp = entry.path().display().to_string();
            if !tmp.contains('$') {
                task_holder.push(tokio::spawn(walk_deeper(sender.clone(),tmp, op_mode)))
            }
        }
    });
    let mut any_dir: Vec<GitDir> = vec![];
    futures::future::join_all(task_holder)
        .await
        .iter()
        .filter_map( |future| {
            if let Ok(f) = future {
                Some(f)
            } else {
                None
            }
        } )
        .for_each( |res| {
            res.to_vec().iter().for_each(|item| {
                if !any_dir.contains(item) {
                    any_dir.push(item.clone())
                }
            })
        });
    any_dir
}