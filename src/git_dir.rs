use std::fmt::{Display, Formatter};
use std::path::{MAIN_SEPARATOR, Path, PathBuf};
use async_recursion::async_recursion;
use serde::de::Error;

use tokio::fs::File;
use tokio::io::{AsyncReadExt};

use crate::api_communicator::{communicate, get_readme_template};
use crate::{ask_a_question, git_dir, read_input};
use crate::github_license::GithubLicense;
use crate::operating_mode::OperatingMode;
use crate::output_printer::PrintMode;
use crate::settings_file::ProgramSettings;

static README_VARIANTS: [&str; 6] = ["README", "README.md", "README.MD", "readme.md", "Readme.md", "Readme.MD"];
static LICENSE_VARIANTS: [&str; 3] = ["LICENSE", "license", "License"];

static DEFAULT_LICENSE_FILE: &str = "LICENSE";
static DEFAULT_README_FILE: &str = "README.md";

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct GitDir {
    pub(crate) path: String,
    pub(crate) readme_path: Option<PathBuf>,
    pub(crate) license_path: Option<PathBuf>,
    pub(crate) project_title: String,
    pub(crate) license: Option<GithubLicense>
}

impl GitDir {
    pub fn init(path: String, license: Option<Vec<GithubLicense>>) -> Self {
        let clean_path = path.replace(format!("{}.git", MAIN_SEPARATOR).as_str(), "");
        let project_title = clean_path.split(MAIN_SEPARATOR).last().unwrap().to_string();

        let mut readme_path: Option<PathBuf> = None;
        let mut license_path: Option<PathBuf> = None;

        README_VARIANTS.into_iter().for_each(|readme_name| {
            if readme_path.is_none() {
                let temp_pth = format!("{}{}{}", &clean_path, MAIN_SEPARATOR, readme_name);
                readme_path = if Path::new(temp_pth.as_str()).exists() { Some(temp_pth.into()) } else { None };
            }
        });

        //TODO find a better and elegant way for this.

        LICENSE_VARIANTS.into_iter().for_each(|license_name| {
            if license_path.is_none() {
                let temp_pth = format!("{}{}{}", &clean_path, MAIN_SEPARATOR, license_name);
                license_path = if Path::new(temp_pth.as_str()).exists() { Some(temp_pth.into()) } else { None };
            }
        });

        

        Self {
            path: clean_path,
            readme_path,
            license_path,
            project_title,
            license: None
        }
    }

    pub fn get_default_readme_path(&self) -> String {
        format!("{}{}{}", self.path, MAIN_SEPARATOR, DEFAULT_README_FILE)
    }

    pub fn get_default_license_path(&self) -> String {
        format!("{}{}{}", self.path, MAIN_SEPARATOR, DEFAULT_LICENSE_FILE)
    }

    pub async fn set_dummy_readme(&mut self, program_settings: &ProgramSettings, print_mode: &mut PrintMode) {
        if self.readme_path.is_none() {
            let dummy_path = self.get_default_readme_path();
            if let Some(readme) = get_readme_template(program_settings, &self.clone()).await {
                if let Err(error) = tokio::fs::write(&dummy_path, readme).await {
                    print_mode.error_msg("Failure during README file creation");
                    print_mode.error_msg(error);
                }
            } else {
                print_mode.error_msg("Failure during README file content creation");
            }
            self.license_path = Some(PathBuf::from(dummy_path));
        }
    }

    async fn replace_in_readme(&self, license: &GithubLicense, pm: &mut PrintMode, multi_license: bool) {
        // declaring placeholders outside of "if let" scope
        let mut new_file_content = String::new();
        let mut new_license_section = String::new();

        // Open Readme file or print error
        if let Ok(mut file_content) = File::open(&self.readme_path.clone().unwrap()).await {
            // placeholder for old file content
            let mut old_file_content = String::new();

            // read old filecontent to string

            if file_content.read_to_string(&mut old_file_content).await.is_ok() {

                // Split file into slices of strings
                let slices_of_old_file = &mut old_file_content.split_inclusive("##").collect::<Vec<&str>>();

                // check if there is a License section
                if let Some(index_of_license) = slices_of_old_file.iter().position(|&c| {
                    c.contains(" License ") || c.contains(" LICENSE ") || c.contains(" License\n") || c.contains(" LICENSE\n")
                }) {

                    //TODO Implement a "is like" function to make this block even more accurate. - Works best when there is a "License" inside of the GitDir struct

                    // Then replace it
                    if let Some(content) = slices_of_old_file.last() {
                        if multi_license {
                            if content == &slices_of_old_file[index_of_license] {
                                new_license_section = [slices_of_old_file[index_of_license], "\n", &license.get_markdown_license_link()].concat()
                            } else {
                                new_license_section = [slices_of_old_file[index_of_license].replace("##", "").as_str(), "\n", &license.get_markdown_license_link(), "\n\n##"].concat()
                            }
                        } else if content == &slices_of_old_file[index_of_license] {
                            new_license_section = [" License\n", &license.get_markdown_license_link()].concat()
                        } else {
                            new_license_section = [" License\n", &license.get_markdown_license_link(), "\n\n##"].concat()
                        }
                    }
                    slices_of_old_file[index_of_license] = &new_license_section;
                }

                // Rebuild the new file from the slices
                for slice in slices_of_old_file {
                    new_file_content = new_file_content + slice;
                }

                // Then overwrite the License file or print message on error
                match tokio::fs::write(self.readme_path.clone().unwrap(), new_file_content).await {
                    Ok(_) => {
                        pm.verbose_msg(format!("Success in overwriting {}", self.readme_path.clone().unwrap().display()), None)
                    }
                    Err(msg) => pm.error_msg(format!(
                        "{} occurred while writing {}",
                        msg,
                        self.readme_path.clone().unwrap().display()
                    )),
                }
            }
        } else if let Err(err) = File::open(self.readme_path.clone().unwrap()).await {
            pm.error_msg(format!(
                "{} occurred while opening file: {}",
                err,
                self.readme_path.clone().unwrap().display()
            ))
        }
    }


    #[async_recursion]
    async fn write_license(&mut self, program_settings: &ProgramSettings, print_mode: &mut PrintMode, user_choice: &GithubLicense, multi_license: bool) {
        if let Some(license_path) = &self.license_path {
                if let Err(error) = tokio::fs::write(license_path, user_choice.clone().set_username_and_year().body).await {
                    print_mode.error_msg(error);
                }

                if self.readme_path.is_some() {
                    if multi_license {
                        self.replace_in_readme(user_choice, print_mode, false).await;
                    }
                } else if ask_a_question("Found no README file - do you want to create one?") {
                    self.set_dummy_readme(program_settings, print_mode).await;
                    self.replace_in_readme(user_choice, print_mode, multi_license).await;
                }
            }
         else if !multi_license {
            print_mode.error_msg("Wanted to set a license, while a license was detected! Use the \"AppendLicenseMode\" for this!");
        } else {
             self.license_path = Some(self.get_default_license_path().into());
             self.license = Some(user_choice.to_owned());
             self.write_license(program_settings, print_mode, user_choice, multi_license).await
         }
    }

    fn list_licenses_and_get_user_input(licenses: &Vec<GithubLicense>) -> Result<usize, Box<dyn std::error::Error>>{
        licenses.iter().enumerate()
            .for_each(|(c, l)| {
                println!("[{}] {}", c + 1, l.name)
            }
            );
        Ok(read_input("Your Selection: ").parse::<usize>()?-1)
    }

    pub async fn execute_user_action(&mut self, program_settings: &ProgramSettings, print_mode: &mut PrintMode, op_mode: &OperatingMode, licenses: Vec<GithubLicense>) {

        if let Ok(uint) = GitDir::list_licenses_and_get_user_input(&licenses) {
            let user_choice = &licenses[uint];
            match op_mode {
                OperatingMode::SetNewLicense => {
                    self.write_license(program_settings, print_mode, &user_choice, false).await
                }
                OperatingMode::AppendLicense => {
                    let mut license_path = self.license_path.clone().unwrap();
                    if license_path.exists() {
                        license_path.set_file_name(format!("{}-{}", DEFAULT_LICENSE_FILE, user_choice.spdx_id))
                    }
                    self.license_path = Some(license_path);
                    self.write_license(program_settings, print_mode, &user_choice, true).await
                }
                OperatingMode::LicenseReplace => {
                    if self.license_path.is_some() && tokio::fs::remove_file(self.license_path.clone().unwrap()).await.is_err() {
                        print_mode.error_msg("Error occurred while deleting the current LICENSE file!");
                        return;
                    }
                    self.write_license(program_settings, print_mode, &user_choice, false).await
                }
                OperatingMode::ShowAllGitDirs => {}
            }
        }
    }
}

impl Display for GitDir {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // TODO: Make this even more prettier
        writeln!(f, "\nProject: {}\nPath: {}\n", self.project_title, self.path)
    }
}
