use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;
use tokio::spawn;

use walkdir::WalkDir;

use crate::licences::*;
use crate::{clear_term, PrintMode, read_input};
use crate::api_communicator::communicate;
use crate::github_license::GithubLicense;
use crate::operating_mode::OperatingMode;
use crate::settings_file::ProgramSettings;


// The Functions returns the name of the directory, where the ".git" folder is contained as Project Title
fn get_project_title(path: &str, pm: &PrintMode) -> String {
    let mut path_splitter = '/';
    if cfg!(windows) {
        path_splitter = '\\'
    }
    if path.len() >= 2 {
        let split: Vec<&str> = path.split(path_splitter).collect();
        pm.verbose_msg(format!("Project title is: {}", split[split.len() - 2]), None);
        split[split.len() - 2].to_string()
    } else {
        "project-title".to_string()
    }
}

// Writes the License file to disk
fn write_license_file(
    license_path: &mut PathBuf,
    license_and_link: &GithubLicense,
    pm: &mut PrintMode,
) {
    // Because the function also can be called through the "AppendLicense" mode - it checks if
    // a file already exists
    if license_path.exists() {
        // If it does, delete the "LICENSE" part from the Path
        license_path.pop();
        // And add "LICENSE-1" or other
        license_path.push(["LICENSE-", &license_and_link.spdx_id].concat());
    }
    // Then write the file and check for errors
    match std::fs::write(&license_path, &license_and_link.body) {
        Ok(_) => pm.verbose_msg(format!("created {}!", license_path.display()), None),
        Err(msg) => {
            pm.error_msg(format!(
                "{} occurred while creating file {}",
                msg,
                license_path.display()
            ));
        }
    }
}

// Writes the Readme file
fn write_readme(readme_path: &PathBuf, current_dir: &str, pm: &mut PrintMode) {
    let project_title = get_project_title(current_dir, pm);
    if File::create(readme_path).is_ok() {
        match std::fs::write(readme_path, readme(project_title)) {
            Ok(_) => {
                pm.verbose_msg(format!("created {}!", readme_path.display()), None);
                pm.verbose_msg("This is a dummy readme and should be replaced!", None)
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

// Appends text to the Readme File

// This still just appends to the end of the file.
// This behavior will be changed in the future.
fn append_to_readme(
    readme_path: &PathBuf,
    license: &GithubLicense,
    pm: &mut PrintMode,
) {
    // Open file in append mode
    if let Ok(mut file) = OpenOptions::new().append(true).open(readme_path) {
        pm.verbose_msg(format!(
            "{:#?} successfully opened in append mode",
            readme_path
        ), None);
        // Write License Link to the File
        match file.write_all(["\n", &license.name].concat().as_bytes()) {
            Ok(_) => pm.verbose_msg(format!("Appended {} to README.md", &license.name), None),
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

// This Function reads the complete "README",
// Tries to filter the "##README" section and replaces it with the
// correct one.
// fn replace_in_readme(
//     readme_path: &PathBuf,
//     license: &GithubLicense,
//     pm: &mut PrintMode,
// ) {
//     // declaring placeholders outside of "if let" scope
//     let mut new_file_content = String::new();
//     let mut new_license_section = String::new();
//
//     // Open Readme file or print error
//     if let Ok(mut file_content) = File::open(readme_path) {
//         // placeholder for old file content
//         let mut old_file_content = String::new();
//
//         // read old filecontent to string
//         if file_content.read_to_string(&mut old_file_content).is_ok() {
//
//             // Split file into slices of strings
//             let slices_of_old_file = &mut old_file_content.split_inclusive("##").collect::<Vec<&str>>();
//
//             // check if there is a License section
//             if let Some(index_of_license) = slices_of_old_file.iter().position(|&c| {
//                 c.contains(" License ") || c.contains(" LICENSE ") || c.contains(" License\n") || c.contains(" LICENSE\n")
//             }) {
//
//                 // Then replace it
//                 if let Some(content) = slices_of_old_file.last() {
//                     if content == &slices_of_old_file[index_of_license] {
//                         new_license_section = [" License\n", &license.name].concat()
//                     } else {
//                         new_license_section = [" License\n", &license.name, "\n\n##"].concat()
//                     }
//                 }
//                 slices_of_old_file[index_of_license] = &new_license_section;
//             }
//
//             // Rebuild the new file from the slices
//             for slice in slices_of_old_file {
//                 new_file_content = new_file_content + slice;
//             }
//
//             // Then overwrite the License file or print message on error
//             match std::fs::write(readme_path, new_file_content) {
//                 Ok(_) => {
//                     pm.verbose_msg(format!("Success in overwriting {}", readme_path.display()), None)
//                 }
//                 Err(msg) => pm.error_msg(format!(
//                     "{} occurred while writing {}",
//                     msg,
//                     readme_path.display()
//                 )),
//             }
//         }
//     } else if let Err(err) = File::open(readme_path) {
//         pm.error_msg(format!(
//             "{} occurred while opening file: {}",
//             err,
//             readme_path.display()
//         ))
//     }
// }

// Deletes all License files in a directory

async fn delete_license_files(path: &mut PathBuf, pm: &mut PrintMode) {
    path.pop(); // remove ".git" from the Path
    let mut vec: Vec<String> = Vec::new(); // Placeholder

    // Iterate over every item inside the directory
    WalkDir::new(&path).max_depth(1).into_iter().filter_map(|files| {
        if let Ok(dir_entry) = files {
            if dir_entry.path().display().to_string().contains("LICENSE") || dir_entry.path().display().to_string().contains("License") {
                // If there is a license file, add it to the collection
                Some(dir_entry.path().display().to_string())
            } else {
                None
            }
        } else {
            None
        }
    }).for_each(|i| vec.push(i));

    // Iterate over the vector, containing the absolute path to the license file and delete it.
    for file in vec {
        match tokio::fs::remove_file(&file).await {
            Ok(_) => {pm.verbose_msg(format!("Deleted: {}", &file), None)}
            Err(msg) => {pm.error_msg(format!("{} occurred \nduring deletion of {}", msg, file))}
        }
    }

    path.push("../LICENSE_old"); // Add "LICENSE" to the Path
}

// This function "does the actual thing"
// It takes the paths, that the user wants to modify as Input
// And returns the number of directories processed

pub async fn insert_license(
    mut paths: Vec<&String>,
    operating_mode: &OperatingMode,
    pm: &mut PrintMode,
) -> usize {
    clear_term();
    // Ask the user, which license he wants to give ANY of the directories
    let license = communicate(&ProgramSettings::init(pm).await).await.unwrap().set_username_and_year();
    // count the items of the paths vector
    let i = &paths.len();
    clear_term();
    // For verbose mode, print every directory the user has chosen
    paths.iter_mut().for_each(|path| pm.verbose_msg(format!("Chosen path(s): {}", path.replace(".git", "")), None));

    // Here begins the real work
    paths.into_iter().for_each(|dir| {
        pm.verbose_msg(format!("Processing dir: {}", dir.replace(".git", "")), None);

        // Create a Path, that points to the README.md file of the directory
        let readme_path: PathBuf = dir.replace(".git", "README.md").into();
        // Create a Path, that points to the LICENSE file of the directory
        let mut license_path: PathBuf = dir.replace(".git", "LICENSE").into();

        // If there is no Readme
        if !readme_path.exists() {
            // Create readme and license
            pm.verbose_msg("README.md not found", None);
            // create the Readme
            write_readme(&readme_path, dir, pm);
            // create the License file
            write_license_file(&mut license_path, &license, pm);
            // append the License link to the License file
            append_to_readme(&readme_path, &license, pm);
        } else if operating_mode == &OperatingMode::AppendLicense || readme_path.exists() {
            // Append Mode
            pm.verbose_msg("README.md found! Appending license....", None);
            write_license_file(&mut license_path, &license, pm);
            append_to_readme(&readme_path, &license, pm)
        } else if operating_mode == &OperatingMode::LicenseReplace {
            // Replace mode
            pm.verbose_msg("README.md found! Replacing license....", None);
            delete_license_files(&mut license_path, pm);
            write_license_file(&mut license_path, &license, pm);
            //replace_in_readme(&readme_path, &license, pm)
        }
    });
    // return count of paths processed
    *i
}
