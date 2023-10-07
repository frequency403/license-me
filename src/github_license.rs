use chrono::prelude::*;
use serde::{Deserialize, Serialize};

use crate::read_input;

#[derive(Serialize, Deserialize, Clone)]
pub struct MiniGithubLicense {
    pub(crate) key: String,
    pub(crate) name: String,
    pub(crate) spdx_id: String,
    pub(crate) url: String,
    pub(crate) node_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct GithubLicense {
    pub(crate) key: String,
    pub(crate) name: String,
    pub(crate) spdx_id: String,
    pub(crate) url: String,
    pub(crate) node_id: String,
    pub(crate) html_url: String,
    pub(crate) description: String,
    pub(crate) implementation: String,
    pub(crate) permissions: Vec<String>,
    pub(crate) conditions: Vec<String>,
    pub(crate) limitations: Vec<String>,
    pub(crate) body: String,
    pub(crate) featured: bool,
}

impl GithubLicense {
    pub fn set_username_and_year(mut self) -> Self {
        if self.body.contains("[fullname]") {
            self.body = self.body.replace(
                "[fullname]",
                read_input("Enter your full name (John Doe): ").as_str(),
            );
        }
        if self.body.contains("[year]") {
            self.body = self
                .body
                .replace("[year]", Utc::now().year().to_string().as_str());
        }
        self
    }

    pub fn get_markdown_license_link(&self) -> String {
        format!("\n[{}]({})", self.spdx_id, self.html_url)
    }

    pub fn list_licenses_and_get_user_input(
        licenses: &[GithubLicense],
    ) -> Result<usize, Box<dyn std::error::Error>> {
        licenses
            .iter()
            .enumerate()
            .for_each(|(c, l)| println!("[{}] {}", c + 1, l.name));
        Ok(read_input("Your Selection: ").parse::<usize>()? - 1)
    }
}
