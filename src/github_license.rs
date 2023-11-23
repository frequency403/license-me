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
    /// Sets the username and year in the `body` field of the struct.
    ///
    /// If the `body` field contains the string `"[fullname]"`, it prompts the user to enter
    /// their full name and replaces `"[fullname]"` with the entered value in the `body` field.
    ///
    /// If the `body` field contains the string `"[year]"`, it replaces `"[year]"` with the
    /// current year in the `body` field.
    ///
    /// Returns `self` after modifying the `body` field.
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

    /// Returns a markdown license link.
    ///
    /// # Returns
    ///
    /// A `String` representing the markdown license link.
    ///
    /// # Examples
    ///
    /// ```
    /// let license = License {
    ///     spdx_id: "MIT".to_string(),
    ///     html_url: "https://opensource.org/licenses/MIT".to_string(),
    /// };
    /// let link = license.get_markdown_license_link();
    /// assert_eq!(link, "\n[MIT](https://opensource.org/licenses/MIT)");
    /// ```
    pub fn get_markdown_license_link(&self) -> String {
        format!("\n[{}]({})", self.spdx_id, self.html_url)
    }

    /// Prints a list of licenses and gets user input for selecting a license.
    ///
    /// # Arguments
    ///
    /// * `licenses` - A slice of `GithubLicense` structs representing the available licenses.
    ///
    /// # Returns
    ///
    /// * `Result<usize, Box<dyn std::error::Error>>` - The index of the selected license.
    ///
    /// # Errors
    ///
    /// This function can return errors if there is an issue with parsing user input or the input is out of range.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::error::Error;
    ///
    /// struct GithubLicense {
    ///     name: String,
    ///     // other fields
    /// }
    ///
    /// fn read_input(prompt: &str) -> String {
    ///     // implementation
    ///     String::from("1")
    /// }
    ///
    /// # pub fn list_licenses_and_get_user_input(licenses: &[GithubLicense]) -> Result<usize, Box<dyn Error>> {
    /// #     licenses
    /// #         .iter()
    /// #         .enumerate()
    /// #         .for_each(|(c, l)| println!("[{}] {}", c + 1, l.name));
    /// #     Ok(read_input("Your Selection: ").parse::<usize>()? - 1)
    /// # }
    /// ```
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
