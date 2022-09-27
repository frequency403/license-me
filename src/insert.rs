use std::fs::File;
use std::path::PathBuf;
use log::{error, info};
use crate::licences::readme;

#[allow(dead_code)]
pub fn get_license_ver() /*-> String*/ {
    match crate::read_input("Choose from Available Licenses: \n\n[1] Apache\n[2]XXX\n[3]XXX").trim() {
        "1" => {}
        "2" => {}
        "3" => {}
        _ => {
            error!("Unknown input!");
            get_license_ver()
        }
    }
}

fn get_project_title(path: &str) -> String{
    let split: Vec<&str> = path.split('\\').collect();
    split[split.len() -2 ].to_string()
}

pub fn insert_license(mut paths: Vec<&String>) -> usize {
    let i = &paths.len();
    paths.iter_mut().for_each(|path| {info!("Chosen path(s): {}", path)});
    paths.into_iter().for_each(|dir| {
        info!("Processing dir: {dir}");
        let readme_path: PathBuf = dir.replace(".git", "README.md").into();
        let license_path: PathBuf = readme_path.display().to_string().replace("README.md", "LICENSE").into();
        let project_title = get_project_title(dir);
        if !readme_path.exists() {
            info!("README.md not found");
            if File::create(&readme_path).is_ok() {
                if std::fs::write(&readme_path, readme(project_title)).is_ok() {
                    info!("created {}!", readme_path.display());
                    info!("This is a dummy readme and should be replaced!")
                } else {
                    error!("Something went wrong creating the File!");
                }
            }
            if std::fs::write(&license_path, "").is_ok() {
                info!("created {}!", license_path.display())
            } else {
                error!("Something went wrong creating the File!");
            }
        } else if readme_path.exists() {
            info!("README.md found! Appending license....");
            if std::fs::write(&license_path, "").is_ok() {
                info!("created {}!", license_path.display())
            } else {
                error!("Something went wrong creating the File!");
            }

            //Add LICENSE and README.md if not available
            //OR Append the Licence link in Markdown to README.md if it is present and then add the license.
        }
        });
    *i
}