use std::fmt::{Display, Formatter};
use std::path::{MAIN_SEPARATOR, Path};
use futures::io::Write;
use tokio::fs::{File, OpenOptions};
use tokio::io::AsyncWriteExt;
use crate::api_communicator::{communicate, get_readme_template};
use crate::operating_mode::OperatingMode;
use crate::output_printer::PrintMode;
use crate::settings_file::ProgramSettings;

static README_VARIANTS: [&str; 6] = ["README", "README.md", "README.MD", "readme.md", "Readme.md", "Readme.MD"];
static LICENSE_VARIANTS: [&str; 3] = ["LICENSE", "license", "License"];

#[derive(Clone, Eq, PartialEq)]
pub struct GitDir {
    pub(crate) path: String,
    pub(crate) has_areadme: bool,
    pub(crate) readme_path: Option<&'static Path>,
    pub(crate) has_alicense: bool,
    pub(crate) license_path: Option<&'static Path>,
    pub(crate) project_title: String,
}

impl GitDir {
    pub fn init(path: String) -> Self {
        let clean_path = path.replace(format!("{}.git", MAIN_SEPARATOR).as_str(), "");
        let project_title = clean_path.split(MAIN_SEPARATOR).last().unwrap().to_string();

        let mut has_readme = false;
        let mut has_license = false;
        let mut readme_path: Option<&Path> = None;
        let mut license_path: Option<&Path> = None;

        for variant in README_VARIANTS {
            if !has_readme {
                let temp_pth = format!("{}{}{}", clean_path, MAIN_SEPARATOR, variant);
                has_readme = Path::new(temp_pth.clone().as_str()).exists();
                readme_path = if has_readme { Some(Path::new(temp_pth.as_str())) } else { None };
            }
        }

        for variant in LICENSE_VARIANTS {
            if !has_license {
                let temp_pth = format!("{}{}{}", &clean_path, MAIN_SEPARATOR, variant);
                has_license = Path::new(temp_pth.clone().as_str()).exists();
                license_path = if has_license { Some(Path::new(temp_pth.as_str())) } else { None };
            }
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

    pub async fn set_dummy_readme(&self, program_settings: &ProgramSettings, print_mode: &mut PrintMode) {
        if !self.has_areadme {
            if let Some(template) = get_readme_template(program_settings).await {
                if tokio::fs::write(self.readme_path.clone().unwrap(), template.replace(&program_settings.replace_in_readme_phrase, &self.project_title)).await.is_err() {
                    print_mode.error_msg("Error Writing License File");
                }
            } else { print_mode.error_msg("Error Writing License File"); }
        }
    }


    pub async fn write_license(&self, program_settings: &ProgramSettings, print_mode: &mut PrintMode, op_mode: &OperatingMode) {

        match op_mode {
            OperatingMode::SetNewLicense => {
                if !self.has_alicense {
                    if let Some(license_path) = self.license_path{
                        if let Some(user_choice) = communicate(program_settings).await {



                            if self.has_areadme {
                                // Open file in append mode
                                if let Ok(mut file) = OpenOptions::new().append(true).open(self.readme_path.unwrap()).await {
                                    print_mode.verbose_msg(format!(
                                        "{:#?} successfully opened in append mode",
                                        self.readme_path
                                    ), None);
                                    // Write License Link to the File

                                        match file.write(["\n", &user_choice.name].concat().as_bytes()).await {
                                        Ok(_) => print_mode.verbose_msg(format!("Appended {} to README.md", &user_choice.name), None),
                                        Err(msg) => print_mode.error_msg(format!(
                                            "{} while appending to file {}",
                                            msg,
                                            self.readme_path.unwrap().display()
                                        )),
                                    }
                                } else {
                                    print_mode.error_msg("Error opening the file in append mode")
                                }
                            }
                        }
                    }
                } else {
                    print_mode.error_msg("");
                }
            }
            OperatingMode::AppendLicense => {}
            OperatingMode::LicenseReplace => {}
            OperatingMode::ShowAllGitDirs => {return}
        }


    }
}

impl Display for GitDir {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "\n{}\nPath: {}\nLicense: {} | Readme: {}", self.project_title, self.path, self.has_alicense, self.has_areadme)
    }
}
