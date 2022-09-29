use crate::licences::*;
use crate::{clear_term, read_input};
use log::{error, info};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

fn get_project_title(path: &str) -> String {
    let split: Vec<&str> = path.split('\\').collect();
    split[split.len() - 2].to_string()
}

pub fn get_license_ver() -> (String, String) {
    let username = read_input("Enter your full name (John Doe): ");
    match read_input(
        "Choose from Popular available Licenses for ALL chosen directories: \n\n\
                [1] MIT License (SPDX short identifier: MIT)\n\
                [2] Apache License, Version 2.0 (SPDX short identifier: Apache-2.0)\n\
                [3] The 3-Clause BSD License (SPDX short identifier: BSD-3-Clause)\n\
                [4] The 2-Clause BSD License (SPDX short identifier: BSD-2-Clause)\n\
                [5] GNU General Public License version 2 (SPDX short identifier: GPL-2.0)\n\
                [6] GNU General Public License version 3 (SPDX short identifier: GPL-3.0)\n\
                [7] GNU Library General Public License, version 2 (SPDX short identifier: LGPL-2.0)\n\
                [8] GNU Lesser General Public License, version 2.1 (SPDX short identifier: LGPL-2.1)\n\
                [9] GNU Lesser General Public License, version 3 (SPDX short identifier: LGPL-3.0)\n\
                [10] Mozilla Public License 2.0 (SPDY short identifier: MPL-2.0)\n\
                [11] Common Development and Distribution License 1.0 (SPDX short identifier: CDDL-1.0)\n\
                [12] Eclipse Public License version 2.0 (SPDX short identifier: EPL-2.0)\n\n\
                Your Selection: ",
    )
    .trim()
    {
        "1" => { mit(username) },
        "2" => { apache2(username) },
        "3" => { bsd3(username) },
        "4" => {bsd2(username)},
        "5" => {gpl2(username)},
        "6" => {gpl3(username)},
        "7" => {lgpl20(username)},
        "8" => {lgpl21(username)},
        "9" => {lgpl30(username)},
        "10" => {mpl2(username)},
        "11" => {cddl(username)},
        "12" => {epl(username)},
        _ => {
            error!("Unknown or wrong input! Remember: One license per Run!");
            get_license_ver()
        }
    }
}

fn write_license_file(license_path: &PathBuf, license_and_link: &(String, String)) {
    if std::fs::write(license_path, &license_and_link.0).is_ok() {
        info!("created {}!", license_path.display())
    } else {
        error!("Something went wrong creating the File!");
    }
}
fn write_readme(readme_path: &PathBuf, current_dir: &str) {
    let project_title = get_project_title(current_dir);
    if File::create(&readme_path).is_ok() {
        if std::fs::write(&readme_path, readme(project_title)).is_ok() {
            info!("created {}!", readme_path.display());
            info!("This is a dummy readme and should be replaced!")
        } else {
            error!("Something went wrong creating the File!");
        }
    }
}
fn append_to_readme(readme_path: &PathBuf, license_and_link: &(String, String)) {
    if let Ok(mut file) = OpenOptions::new().append(true).open(readme_path) {
        info!("{:#?} successfully opened in append mode", readme_path);
        if file
            .write_all(["\n", &license_and_link.1].concat().as_bytes())
            .is_ok()
        {
            info!("Appended {} to README.md", &license_and_link.1)
        } else {
            error!("Error while appending to file!")
        }
    } else {
        error!("Error opening the file in append mode")
    }
}
pub fn insert_license(mut paths: Vec<&String>) -> usize {
    clear_term();
    let license = get_license_ver();
    clear_term();
    let i = &paths.len();
    paths
        .iter_mut()
        .for_each(|path| info!("Chosen path(s): {}", path));
    paths.into_iter().for_each(|dir| {
        info!("Processing dir: {dir}");
        let readme_path: PathBuf = dir.replace(".git", "README.md").into();
        let license_path: PathBuf = readme_path
            .display()
            .to_string()
            .replace("README.md", "LICENSE")
            .into();
        if !readme_path.exists() {
            info!("README.md not found");
            write_readme(&readme_path, dir);
            write_license_file(&license_path, &license);
            append_to_readme(&readme_path, &license);
        } else if readme_path.exists() {
            info!("README.md found! Appending license....");
            write_license_file(&license_path, &license);
            append_to_readme(&readme_path, &license)
        }
    });
    *i
}
