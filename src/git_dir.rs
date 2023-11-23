use std::fmt::{Display, Formatter};
use std::path::{MAIN_SEPARATOR, Path, PathBuf};

use async_recursion::async_recursion;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

use crate::ask_a_question;
use crate::alike::is_alike;
use crate::api_communicator::get_readme_template;
use crate::github_license::GithubLicense;
use crate::operating_mode::OperatingMode;
use crate::output_printer::PrintMode;
use crate::settings_file::ProgramSettings;

static README_VARIANTS: [&str; 6] = [
    "README",
    "README.md",
    "README.MD",
    "readme.md",
    "Readme.md",
    "Readme.MD",
];
static LICENSE_VARIANTS: [&str; 3] = ["LICENSE", "license", "License"];

static DEFAULT_LICENSE_FILE: &str = "LICENSE";
static DEFAULT_README_FILE: &str = "README.md";

/// Represents a directory containing a Git repository.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct GitDir {
    pub(crate) path: String,
    pub(crate) readme_path: Option<PathBuf>,
    pub(crate) license_path: Option<PathBuf>,
    pub(crate) project_title: String,
    pub(crate) license: Option<GithubLicense>,
}

impl GitDir {
    /// Initializes a new instance of `Self` with the given path and optional licenses.
    ///
    /// # Arguments
    ///
    /// * `path` - A string representing the path to the project.
    /// * `licenses` - An optional vector of `GithubLicense` representing the available licenses.
    ///
    /// # Returns
    ///
    /// A new instance of `Self`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tokio::fs::read_to_string;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let path = String::from("path/to/project");
    ///     let licenses = vec![GithubLicense { body: String::from("MIT License") }];
    ///
    ///     let result = init(path, Some(licenses)).await;
    ///     assert_eq!(result.path, "path/to/project");
    ///     assert_eq!(result.readme_path, None);
    ///     assert_eq!(result.license_path, None);
    ///     assert_eq!(result.project_title, "project");
    ///     assert_eq!(result.license, None);
    /// }
    /// ```
    pub async fn init(path: String, licenses: Option<Vec<GithubLicense>>) -> Self {
        let clean_path = path.replace(format!("{}.git", MAIN_SEPARATOR).as_str(), "");
        let project_title = clean_path.split(MAIN_SEPARATOR).last().unwrap().to_string();

        let mut readme_path: Option<PathBuf> = None;
        let mut license_path: Option<PathBuf> = None;

        //TODO find a better and elegant way for this block of code.
        README_VARIANTS.into_iter().for_each(|readme_name| {
            if readme_path.is_none() {
                let temp_pth = format!("{}{}{}", &clean_path, MAIN_SEPARATOR, readme_name);
                readme_path = if Path::new(temp_pth.as_str()).exists() {
                    Some(temp_pth.into())
                } else {
                    None
                };
            }
        });

        LICENSE_VARIANTS.into_iter().for_each(|license_name| {
            if license_path.is_none() {
                let temp_pth = format!("{}{}{}", &clean_path, MAIN_SEPARATOR, license_name);
                license_path = if Path::new(temp_pth.as_str()).exists() {
                    Some(temp_pth.into())
                } else {
                    None
                };
            }
        });
        // END block

        let license_holder = if let Some(license_vec) = licenses {
            if let Some(found_license) = &license_path {
                if let Ok(license_content) = tokio::fs::read_to_string(found_license).await {
                    license_vec
                        .into_iter()
                        .filter(|available_licenses| {
                            is_alike(available_licenses.clone().body, license_content.clone(), 60)
                        })
                        .last()
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        Self {
            path: clean_path,
            readme_path,
            license_path,
            project_title,
            license: license_holder,
        }
    }

    /// Returns the default path for the README file.
    ///
    /// # Arguments
    ///
    /// - `self`: The instance of the struct.
    ///
    /// # Returns
    ///
    /// The default path for the README file as a `String`.
    pub fn get_default_readme_path(&self) -> String {
        format!("{}{}{}", self.path, MAIN_SEPARATOR, DEFAULT_README_FILE)
    }

    /// Returns the path of the default license file.
    ///
    /// The default license file path is constructed by concatenating the current
    /// path with the main separator and the name of the default license file.
    ///
    /// # Arguments
    ///
    /// - `self`: A reference to the current object.
    ///
    /// # Returns
    ///
    /// The path of the default license file as a `String`.
    pub fn get_default_license_path(&self) -> String {
        format!("{}{}{}", self.path, MAIN_SEPARATOR, DEFAULT_LICENSE_FILE)
    }

    /// Sets a dummy README file if it does not already exist.
    ///
    /// # Arguments
    ///
    /// * `program_settings` - The program settings.
    /// * `print_mode` - The print mode.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # #[tokio::main]
    /// # async fn main() {
    /// # use std::path::PathBuf;
    /// # use tokio::fs;
    /// # use some_crate::{ProgramSettings, PrintMode};
    /// # struct SomeStruct {
    /// #     readme_path: Option<PathBuf>,
    /// #     license_path: Option<PathBuf>,
    /// # }
    /// # impl SomeStruct {
    /// #     async fn get_readme_template(
    /// #         program_settings: &ProgramSettings,
    /// #         some_param: &SomeStruct,
    /// #     ) -> Option<Vec<u8>> {
    /// #         // Some implementation here
    /// #         Some(vec![1, 2, 3])
    /// #     }
    /// #     fn get_default_readme_path(&self) -> PathBuf {
    /// #         PathBuf::from("some_path")
    /// #     }
    /// pub async fn set_dummy_readme(
    ///     &mut self,
    ///     program_settings: &ProgramSettings,
    ///     print_mode: &mut PrintMode,
    /// ) {
    ///     if self.readme_path.is_none() {
    ///         let dummy_path = self.get_default_readme_path();
    ///         if let Some(readme) = get_readme_template(program_settings, &self.clone()).await {
    ///             if let Err(error) = fs::write(&dummy_path, readme).await {
    ///                 print_mode.error_msg("Failure during README file creation");
    ///                 print_mode.error_msg(error);
    ///             }
    ///         } else {
    ///             print_mode.error_msg("Failure during README file content creation");
    ///         }
    ///         self.license_path = Some(dummy_path);
    ///     }
    /// }
    /// # }
    /// ```
    pub async fn set_dummy_readme(
        &mut self,
        program_settings: &ProgramSettings,
        print_mode: &mut PrintMode,
    ) {
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

    /// Replaces the license section in the readme file with a new license.
    ///
    /// # Arguments
    ///
    /// * `license` - A reference to the `GithubLicense` object representing the new license.
    /// * `pm` - A mutable reference to the `PrintMode` object for printing messages.
    /// * `multi_license` - A boolean value indicating whether the project has multiple licenses.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut printer = PrintMode::new();
    /// let license = GithubLicense::new("MIT", "https://opensource.org/licenses/MIT");
    /// let mut project = Project::new();
    ///
    /// project.replace_in_readme(&license, &mut printer, false).await;
    /// ```
    async fn replace_in_readme(
        &self,
        license: &GithubLicense,
        pm: &mut PrintMode,
        multi_license: bool,
    ) {
        // declaring placeholders outside of "if let" scope
        let mut new_file_content = String::new();
        let mut new_license_section = String::new();

        if let Some(path) = &self.readme_path.clone() {

            // Open Readme file or print error
            if let Ok(mut file_content) = File::open(path).await {
                // placeholder for old file content
                let mut old_file_content = String::new();

                // read old filecontent to string

                if file_content
                    .read_to_string(&mut old_file_content)
                    .await
                    .is_ok()
                {
                    // Split file into slices of strings
                    let slices_of_old_file = &mut old_file_content
                        .split_inclusive("##")
                        .collect::<Vec<&str>>();

                    // check if there is a License section
                    if let Some(index_of_license) = slices_of_old_file.iter().position(|&c| {
                        c.contains(" License ")
                            || c.contains(" LICENSE ")
                            || c.contains(" License\n")
                            || c.contains(" LICENSE\n")
                    }) {
                        //TODO Implement a "is like" function to make this block even more accurate. - Works best when there is a "License" inside of the GitDir struct

                        // Then replace it
                        if let Some(content) = slices_of_old_file.last() {
                            if multi_license {
                                if content == &slices_of_old_file[index_of_license]
                                    || is_alike(content, &slices_of_old_file[index_of_license], 70)
                                {
                                    new_license_section = [
                                        slices_of_old_file[index_of_license],
                                        "\n",
                                        &license.get_markdown_license_link(),
                                    ]
                                        .concat()
                                } else {
                                    new_license_section = [
                                        slices_of_old_file[index_of_license]
                                            .replace("##", "")
                                            .as_str(),
                                        "\n",
                                        &license.get_markdown_license_link(),
                                        "\n\n##",
                                    ]
                                        .concat()
                                }
                            } else if content == &slices_of_old_file[index_of_license] {
                                new_license_section =
                                    [" License\n", &license.get_markdown_license_link()].concat()
                            } else {
                                new_license_section =
                                    [" License\n", &license.get_markdown_license_link(), "\n\n##"]
                                        .concat()
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
                        Ok(_) => pm.verbose_msg(
                            format!(
                                "Success in overwriting {}",
                                self.readme_path.clone().unwrap().display()
                            ),
                            None,
                        ),
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
    }

    /// Writes the license file.
    ///
    /// # Arguments
    ///
    /// * `program_settings` - The program settings.
    /// * `print_mode` - The print mode.
    /// * `user_choice` - The chosen GitHub license.
    /// * `multi_license` - Indicates whether multiple licenses are used.
    #[async_recursion]
    async fn write_license(
        &mut self,
        program_settings: &ProgramSettings,
        print_mode: &mut PrintMode,
        user_choice: &GithubLicense,
        multi_license: bool,
    ) {
        if self.license_path.is_none() || self.license.is_none() {
            if let Err(error) = tokio::fs::write(
                self.get_default_license_path(),
                user_choice.clone().set_username_and_year().body,
            )
                .await
            {
                print_mode.error_msg(error);
            }
            self.license_path = Some(self.get_default_license_path().into());
            self.license = Some(user_choice.to_owned());
            if self.readme_path.is_some() {
                if multi_license {
                    self.replace_in_readme(user_choice, print_mode, false).await;
                }
            } else if ask_a_question("Found no README file - do you want to create one?") {
                self.set_dummy_readme(program_settings, print_mode).await;
                self.replace_in_readme(user_choice, print_mode, multi_license).await;
            }
        } else {
            self.write_license(program_settings, print_mode, user_choice, multi_license)
                .await
        }
    }


    pub async fn execute_user_action(
        &mut self,
        program_settings: &ProgramSettings,
        print_mode: &mut PrintMode,
        op_mode: &OperatingMode,
        licenses: Vec<GithubLicense>,
    ) {
        if op_mode == &OperatingMode::Unlicense {
            if let Some(unwrapped_license_path) = self.license_path.clone() {
                if let Err(err) = tokio::fs::remove_file(unwrapped_license_path).await {
                    print_mode.error_msg(format!("{} occurred while deleting the license file in Unlicense mode", err))
                } else {
                    self.license_path = None;
                    self.license = None;
                }
            }
            return;
        }

        if let Ok(uint) = GithubLicense::list_licenses_and_get_user_input(&licenses) {
            let user_choice = &licenses[uint];
            match op_mode {
                OperatingMode::SetNewLicense => {
                    self.write_license(program_settings, print_mode, user_choice, false)
                        .await
                }
                OperatingMode::AppendLicense => {
                    let mut license_path = self.license_path.clone().unwrap();
                    if license_path.exists() {
                        license_path.set_file_name(format!(
                            "{}-{}",
                            DEFAULT_LICENSE_FILE, user_choice.spdx_id
                        ))
                    }
                    self.license_path = Some(license_path);
                    self.write_license(program_settings, print_mode, user_choice, true)
                        .await
                }
                OperatingMode::LicenseReplace => {
                    if self.license_path.is_some()
                        && tokio::fs::remove_file(self.license_path.clone().unwrap())
                        .await
                        .is_err()
                    {
                        print_mode
                            .error_msg("Error occurred while deleting the current LICENSE file!");
                        return;
                    }
                    self.write_license(program_settings, print_mode, user_choice, false)
                        .await
                }
                _ => {}
            }
        }
    }
}

impl Display for GitDir {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // TODO: Make this even more prettier
        writeln!(
            f,
            "\nProject: {}\nPath: {}\n",
            self.project_title, self.path
        )
    }
}
