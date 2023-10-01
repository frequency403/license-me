use std::fmt::{Display, format, Formatter};
use std::path::{MAIN_SEPARATOR, MAIN_SEPARATOR_STR, Path, PathBuf};
use futures::io::Write;
use serde::de::Unexpected::Str;
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc::OwnedPermit;
use crate::api_communicator::{communicate, get_readme_template};
use crate::ask_a_question;
use crate::github_license::GithubLicense;
use crate::operating_mode::OperatingMode;
use crate::output_printer::PrintMode;
use crate::settings_file::ProgramSettings;

static README_VARIANTS: [&str; 6] = ["README", "README.md", "README.MD", "readme.md", "Readme.md", "Readme.MD"];
static LICENSE_VARIANTS: [&str; 3] = ["LICENSE", "license", "License"];

static DEFAULT_LICENSE_FILE: &str = "LICENSE";
static DEFAULT_README_FILE: &str = "README.md";

#[derive(Clone, Eq, PartialEq)]
pub struct GitDir {
    pub(crate) path: String,
    pub(crate) has_areadme: bool,
    readme_path: Option<PathBuf>,
    pub(crate) has_alicense: bool,
    license_path: Option<PathBuf>,
    pub(crate) project_title: String,
}

impl GitDir {
    pub fn init(path: String) -> Self {
        let clean_path = path.replace(format!("{}.git", MAIN_SEPARATOR).as_str(), "");
        let project_title = clean_path.split(MAIN_SEPARATOR).last().unwrap().to_string();

        let mut has_readme = false;
        let mut has_license = false;
        let mut readme_path: Option<PathBuf> = None;
        let mut license_path: Option<PathBuf> = None;

        for variant in README_VARIANTS {
            if !has_readme {
                let temp_pth = format!("{}{}{}", &clean_path, MAIN_SEPARATOR, variant);
                has_readme = Path::new(temp_pth.as_str()).exists();
                readme_path = if has_readme { Some(temp_pth.into()) } else { None };
            }
        }

        for variant in LICENSE_VARIANTS {
            if !has_license {
                let temp_pth = format!("{}{}{}", &clean_path, MAIN_SEPARATOR, variant);
                has_license = Path::new(temp_pth.as_str()).exists();
                license_path = if has_license { Some(temp_pth.into()) } else { None };
            }
        }

        if license_path.is_none() {
            license_path = Some(format!("{}{}{}", &clean_path, MAIN_SEPARATOR, DEFAULT_LICENSE_FILE).into());
        }
        if readme_path.is_none() {
            readme_path = Some(format!("{}{}{}", &clean_path, MAIN_SEPARATOR, DEFAULT_README_FILE).into());
        }


        Self {
            path: clean_path,
            has_areadme: has_readme,
            readme_path,
            has_alicense: has_license,
            license_path,
            project_title,
        }
    }

    pub async fn set_dummy_readme(&mut self, program_settings: &ProgramSettings, print_mode: &mut PrintMode) {
        if let Some(readme) = get_readme_template(program_settings, &self.clone()).await {
            if let Err(error) = tokio::fs::write(self.readme_path.clone().unwrap(), readme).await {
                print_mode.error_msg("Failure during README file creation");
                print_mode.error_msg(error);
            }
        } else {
            print_mode.error_msg("Failure during README file content creation");
        }
        if !self.has_areadme {
            self.has_areadme = true;
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

                    // Then replace it
                    if let Some(content) = slices_of_old_file.last() {
                        if multi_license {
                            if content == &slices_of_old_file[index_of_license] {
                                new_license_section = [slices_of_old_file[index_of_license], &license.get_markdown_license_link()].concat()
                            } else {
                                new_license_section = [slices_of_old_file[index_of_license], &license.get_markdown_license_link(), "\n\n##"].concat()
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


    async fn write_license(&mut self, program_settings: &ProgramSettings, print_mode: &mut PrintMode, user_choice: &GithubLicense, multi_license: bool) {
        if !self.has_alicense {
            if let Some(license_path) = &self.license_path {
                if let Err(error) = tokio::fs::write(license_path, user_choice.clone().set_username_and_year().body).await {
                    print_mode.error_msg(error);
                }

                if self.has_areadme {

                    if multi_license {
                        self.replace_in_readme(user_choice, print_mode, false).await;
                    }

                    // // Open file in append mode
                    // if let Ok(mut file) = OpenOptions::new().append(true).open(self.readme_path.clone().unwrap()).await {
                    //     print_mode.verbose_msg(format!(
                    //         "{:#?} successfully opened in append mode",
                    //         self.readme_path.clone().unwrap()
                    //     ), None);
                    //
                    //     // Write License Link to the File
                    //
                    //     match file.write(user_choice.get_markdown_license_link().as_bytes()).await {
                    //         Ok(_) => print_mode.verbose_msg(format!("Appended {} to README.md", &user_choice.name), None),
                    //         Err(msg) => print_mode.error_msg(format!(
                    //             "{} while appending to file {}",
                    //             msg,
                    //             self.readme_path.clone().unwrap().display()
                    //         )),
                    //     }
                    // } else {
                    //     print_mode.error_msg("Error opening the file in append mode")
                    // }
                } else if ask_a_question("Found no README file - do you want to create one?") {
                    self.set_dummy_readme(program_settings, print_mode).await;
                    self.replace_in_readme(user_choice, print_mode, multi_license).await;
                }
            }
        } else if !multi_license {
            print_mode.error_msg("Wanted to set a license, while a license was detected! Use the \"AppendLicenseMode\" for this!");
        }
    }


    pub async fn execute_user_action(&mut self, program_settings: &ProgramSettings, print_mode: &mut PrintMode, op_mode: &OperatingMode) {
        if let Some(user_choice) = communicate(program_settings).await {
            match op_mode {
                OperatingMode::SetNewLicense => {
                    self.write_license(program_settings, print_mode, &user_choice, false).await
                }
                OperatingMode::AppendLicense => {
                    let mut license_path = self.license_path.clone().unwrap();
                    if license_path.exists() {
                        license_path.pop();
                        license_path.push(format!("{}-{}", DEFAULT_LICENSE_FILE, user_choice.spdx_id));
                    }
                    self.license_path = Some(license_path);
                    self.write_license(program_settings, print_mode, &user_choice, true).await
                }
                OperatingMode::LicenseReplace => {
                    if self.has_alicense && tokio::fs::remove_file(self.license_path.clone().unwrap()).await.is_err() {
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
        writeln!(f, "\nProject: {}\nPath: {}\n", self.project_title, self.path)
    }
}
