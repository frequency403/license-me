use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;

use walkdir::WalkDir;

use crate::licences::*;
use crate::{clear_term, read_input, PrintMode};

fn get_project_title(path: &str, pm: &PrintMode) -> String {
    let split: Vec<&str> = path.split('\\').collect();
    pm.verbose_msg(format!("Project title is: {}", split[split.len() - 2]));
    split[split.len() - 2].to_string()
}

fn get_license_ver(pm: &PrintMode) -> (String, String, String) {
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
    ).trim() {
        "1" => { mit(username) }
        "2" => { apache2(username) }
        "3" => { bsd3(username) }
        "4" => { bsd2(username) }
        "5" => { gpl2() }
        "6" => { gpl3() }
        "7" => { lgpl20() }
        "8" => { lgpl21() }
        "9" => { lgpl30() }
        "10" => { mpl2() }
        "11" => { cddl() }
        "12" => { epl() }
        _ => {
            pm.error_msg("Unknown or wrong input! Remember: One license per Run!");
            get_license_ver(pm)
        }
    }
}

fn write_license_file(
    license_path: &mut PathBuf,
    license_and_link: &(String, String, String),
    pm: &PrintMode,
) {
    if license_path.exists() {
        license_path.pop();
        license_path.push(["LICENSE-", &license_and_link.2].concat());
    }
    match std::fs::write(&license_path, &license_and_link.0) {
        Ok(_) => pm.verbose_msg(format!("created {}!", license_path.display())),
        Err(msg) => {
            pm.error_msg(format!(
                "{} occurred while creating file {}",
                msg,
                license_path.display()
            ));
        }
    }
}

fn write_readme(readme_path: &PathBuf, current_dir: &str, pm: &PrintMode) {
    let project_title = get_project_title(current_dir, pm);
    if File::create(&readme_path).is_ok() {
        match std::fs::write(&readme_path, readme(project_title)) {
            Ok(_) => {
                pm.verbose_msg(format!("created {}!", readme_path.display()));
                pm.verbose_msg("This is a dummy readme and should be replaced!")
            }
            Err(msg) => {
                pm.error_msg(format!(
                    "{} occurred while creating file {}",
                    msg,
                    readme_path.display()
                ));
            }
        }
    }
}

fn append_to_readme(
    readme_path: &PathBuf,
    license_and_link: &(String, String, String),
    pm: &PrintMode,
) {
    if let Ok(mut file) = OpenOptions::new().append(true).open(readme_path) {
        pm.verbose_msg(format!(
            "{:#?} successfully opened in append mode",
            readme_path
        ));
        match file.write_all(["\n", &license_and_link.1].concat().as_bytes()) {
            Ok(_) => pm.verbose_msg(format!("Appended {} to README.md", &license_and_link.1)),
            Err(msg) => pm.error_msg(format!(
                "{} while appending to file {}",
                msg,
                readme_path.display()
            )),
        }
    } else {
        pm.error_msg("Error opening the file in append mode")
    }
}

fn replace_in_readme(
    readme_path: &PathBuf,
    license_and_link: &(String, String, String),
    pm: &PrintMode,
) {
    let mut new_file_content = String::new();
    let mut new_license_section = String::new();
    if let Ok(mut file_content) = File::open(readme_path) {
        let mut old_file_content = String::new();
        if file_content.read_to_string(&mut old_file_content).is_ok() {
            let slices_of_old_file = &mut old_file_content
                .split_inclusive("##")
                .collect::<Vec<&str>>();
            if let Some(index_of_license) = slices_of_old_file.iter().position(|&c| {
                c.contains(" License ")
                    || c.contains(" LICENSE ")
                    || c.contains(" License\n")
                    || c.contains(" LICENSE\n")
            }) {
                if let Some(content) = slices_of_old_file.last() {
                    if content == &slices_of_old_file[index_of_license] {
                        new_license_section = [" License\n", &license_and_link.1].concat()
                    } else {
                        new_license_section = [" License\n", &license_and_link.1, "\n\n##"].concat()
                    }
                }
                slices_of_old_file[index_of_license] = &new_license_section;
            }
            for slice in slices_of_old_file {
                new_file_content = new_file_content + slice;
            }
            match std::fs::write(readme_path, new_file_content) {
                Ok(_) => {
                    pm.verbose_msg(format!("Success in overwriting {}", readme_path.display()))
                }
                Err(msg) => pm.error_msg(format!(
                    "{} occurred while writing {}",
                    msg,
                    readme_path.display()
                )),
            }
        }
    } else if let Err(err) = File::open(readme_path) {
        pm.error_msg(format!(
            "{} occurred while opening file: {}",
            err,
            readme_path.display()
        ))
    }
}

fn delete_license_files(path: &mut PathBuf, pm: &PrintMode) {
    path.pop();
    let mut vec: Vec<String> = Vec::new();
    WalkDir::new(&path)
        .into_iter()
        .filter_map(|files| {
            if let Ok(dir_entry) = files {
                if dir_entry.path().display().to_string().contains("LICENSE")
                    || dir_entry.path().display().to_string().contains("License")
                {
                    Some(dir_entry.path().display().to_string())
                } else {
                    None
                }
            } else {
                None
            }
        }).for_each(|i| vec.push(i));
    vec.into_iter()
        .for_each(|file| match std::fs::remove_file(&file) {
            Ok(_) => pm.verbose_msg(format!("Deleted: {}", &file)),
            Err(msg) => pm.error_msg(format!("{} occurred \nduring deletion of {}", msg, file)),
        });
    path.push("LICENSE");
}

pub fn insert_license(
    mut paths: Vec<&String>,
    operating_mode: (bool, bool),
    pm: &PrintMode,
) -> usize {
    clear_term();
    let license = get_license_ver(pm);
    let i = &paths.len();
    clear_term();
    paths
        .iter_mut()
        .for_each(|path| pm.verbose_msg(format!("Chosen path(s): {}", path)));
    paths.into_iter().for_each(|dir| {
        pm.verbose_msg(format!("Processing dir: {}", dir));
        let readme_path: PathBuf = dir.replace(".git", "README.md").into();
        let mut license_path: PathBuf = dir.replace(".git", "LICENSE").into();
        if !readme_path.exists() {
            pm.verbose_msg("README.md not found");
            write_readme(&readme_path, dir, pm);
            write_license_file(&mut license_path, &license, pm);
            append_to_readme(&readme_path, &license, pm);
        } else if operating_mode.0 {
            pm.verbose_msg("README.md found! Appending license....");
            write_license_file(&mut license_path, &license, pm);
            append_to_readme(&readme_path, &license, pm)
        } else if operating_mode.1 {
            pm.verbose_msg("README.md found! Replacing license....");
            delete_license_files(&mut license_path, pm);
            write_license_file(&mut license_path, &license, pm);
            replace_in_readme(&readme_path, &license, pm)
        }
    });
    *i
}
