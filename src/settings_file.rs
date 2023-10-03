use std::fmt::{Display, Formatter};
use std::path::{Path, MAIN_SEPARATOR};
use std::string::ToString;

use serde::{Deserialize, Serialize};

use crate::output_printer::PrintMode;

#[derive(Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProgramSettings {
    pub(super) github_user: String,
    pub(super) github_api_token: Option<String>,
    pub(super) readme_template_link: String,
    pub(super) replace_in_readme_phrase: String,
}

impl Default for ProgramSettings {
    fn default() -> Self {
        Self {
            github_user: "frequency403".to_string(),
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
    pub async fn init(pm: &mut PrintMode) -> Self {
        let settings_file_path = format!(
            "{}{}{}",
            std::env::current_dir().unwrap().display(),
            MAIN_SEPARATOR,
            "settings.json"
        );
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
                    pm.error_msg("Recreated News because of Internal Failure!");
                    remove_and_create.await
                }
            } else {
                pm.verbose_msg("Error reading File contents. Assuming the binary signature is malformed. Recreating the Settings File", None);
                pm.error_msg("Recreated News because of Internal Failure!");

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
}
