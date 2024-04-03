use std::env::consts::OS;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};
use std::string::ToString;

use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;

use crate::output_printer::PrintMode;

#[derive(Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProgramSettings {
    pub(super) github_user: String,
    pub(super) github_api_token: Option<String>,
    pub(super) readme_template_link: String,
    pub(super) replace_in_readme_phrase: String,
}

impl Default for ProgramSettings {
    /// Returns the default configuration for the application.
    ///
    /// The default configuration consists of:
    /// - An empty `github_user` string.
    /// - An optional `github_api_token` that is set to `None`.
    /// - A `readme_template_link` string that is set to "https://raw.githubusercontent.com/PurpleBooth/a-good-readme-template/main/README.md".
    /// - A `replace_in_readme_phrase` string that is set to "# Project Title".
    ///
    /// # Example
    ///
    /// ```
    /// use crate::config::Config;
    ///
    /// let default_config = Config::default();
    ///
    /// assert_eq!(default_config.github_user, "");
    /// assert_eq!(default_config.github_api_token, None);
    /// assert_eq!(default_config.readme_template_link, "https://raw.githubusercontent.com/PurpleBooth/a-good-readme-template/main/README.md");
    /// assert_eq!(default_config.replace_in_readme_phrase, "# Project Title");
    /// ```
    fn default() -> Self {
        Self {
            github_user: String::new(),
            github_api_token: None,
            readme_template_link: "https://raw.githubusercontent.com/PurpleBooth/a-good-readme-template/main/README.md".to_string(),
            replace_in_readme_phrase: "# Project Title".to_string(),
        }
    }
}

impl Display for ProgramSettings {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "\nGithub Username: {}\nGithub API Token: {:?}\nLink to license-template: {}\nGetting replaced in Readme: {}", self.github_user, self.github_api_token, self.readme_template_link, self.replace_in_readme_phrase)
    }
}

impl ProgramSettings {

    fn get_settings_file_path() -> String {
        let mut home_dir = String::from("");
        if OS.to_lowercase().contains("windows"){
            let key_content = std::env::var("APPDATA").unwrap_or(format!("{}", std::env::current_dir().unwrap().display().to_string()));
            home_dir = key_content
        } else {
            let key_content = std::env::var("HOME").unwrap_or(format!("{}", std::env::current_dir().unwrap().display().to_string()));
            home_dir = key_content
        }
        [home_dir.as_str(), "LicenseMe", "settings.json"].iter().collect::<PathBuf>().display().to_string()
    }

    /// Initializes the program settings.
    ///
    /// This function loads the settings from a file on disk, or creates a new settings file if no file is found.
    /// If the file exists and its contents are valid, the settings are loaded from the file.
    /// If the file is missing or the contents are invalid, a new settings file is created.
    ///
    /// # Arguments
    ///
    /// * `Pm` - A mutable reference to the `PrintMode` struct.
    ///
    /// # Returns
    ///
    /// Returns a new instance of `Self` containing the loaded or newly created settings.
    pub async fn init(pm: &mut PrintMode) -> Self {
        let settings_file_path = ProgramSettings::get_settings_file_path();
        let def = Self::default();
        let remove_and_create = async {
            tokio::fs::remove_file(&settings_file_path)
                .await
                .unwrap_or_default();
            tokio::fs::write(
                &settings_file_path,
                serde_json::to_string_pretty(&def).unwrap_or_default(),
            )
                .await
                .unwrap_or_default();
            Self::default()
        };
        pm.verbose_msg("Start loading Settings file", None);
        if Path::exists(settings_file_path.as_ref()) {
            pm.verbose_msg("Found settings file, loading....", None);
            if let Ok(file_contents) = tokio::fs::read_to_string(&settings_file_path).await {
                if let Ok(obj) = serde_json::from_str::<Self>(file_contents.as_str()) {
                    pm.verbose_msg("Content of file is valid.", None);
                    pm.normal_msg("Settings File successfully loaded from disk");
                    obj
                } else {
                    pm.verbose_msg(
                        "Object Deserialization got some errors. Recreating the Settings File",
                        None,
                    );
                    pm.error_msg("Recreated new settings file because of Internal Failure!");
                    remove_and_create.await
                }
            } else {
                pm.verbose_msg("Error reading File contents. Assuming the binary signature is malformed. Recreating the Settings File", None);
                pm.error_msg("Recreated new settings file because of Internal Failure!");

                remove_and_create.await
            }
        } else {
            pm.verbose_msg("No Settings File was present, creating one!", None);
            tokio::fs::write(
                &settings_file_path,
                serde_json::to_string_pretty(&def).unwrap_or_default(),
            )
                .await
                .unwrap_or_default();
            pm.normal_msg("Settings File created");
            Self::default()
        }
    }

    /// Saves the changes made to the program settings.
    ///
    /// This function attempts to open the settings file in write mode, truncate the file if it exists,
    /// and write the serialized program settings into the file.
    ///
    /// # Errors
    ///
    /// If an error occurs during the file operations or serialization, an `Err` variant is returned.
    /// The error message will be "Error writing into settings file!".
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use std::error::Error;
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn Error>> {
    /// #
    /// # struct ProgramSettings;
    /// #
    /// # impl ProgramSettings {
    /// #     fn get_settings_file_path() -> &'static str {
    /// #         unimplemented!()
    /// #     }
    /// # }
    /// #
    /// # let mut program_settings = ProgramSettings;
    /// #
    /// program_settings.save_changes().await?;
    /// #
    /// # Ok(())
    /// # }
    /// ```
    pub async fn save_changes(&mut self) -> Result<(), Box<dyn Error>> {
        let open_opt = tokio::fs::OpenOptions::new()
            .truncate(true)
            .write(true)
            .open(ProgramSettings::get_settings_file_path())
            .await;
        if let Ok(mut file) = open_opt {
            file.write_all(serde_json::to_string(self)?.as_bytes()).await?;
            Ok(())
        } else {
            Err(Box::from("Error writing into settings file!"))
        }
    }
}
