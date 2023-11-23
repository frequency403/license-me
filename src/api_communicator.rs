use std::error::Error;
use std::fmt::{Display, Formatter};

use reqwest::{RequestBuilder, StatusCode};
use reqwest::header::{ACCEPT, AUTHORIZATION, HeaderMap, USER_AGENT};
use serde::{Deserialize, Serialize};

use crate::git_dir::GitDir;
use crate::github_license::{GithubLicense, MiniGithubLicense};
use crate::settings_file::ProgramSettings;

/// Represents an API error.
#[derive(Serialize, Deserialize)]
pub struct ApiError {
    message: String,
    documentation_url: String,
}

impl ApiError {
    /// Formats the error code and canonical reason as a string.
    ///
    /// # Arguments
    ///
    /// * `status` - The `StatusCode` for the error.
    ///
    /// # Returns
    ///
    /// A formatted string with the error code and canonical reason.
    pub fn with_error_code(self, status: StatusCode) -> String {
        format!("\t\tError Code: {}\n\t\tCanonical Reason: {:?}", status.as_str(), status.canonical_reason())
    }
}

impl Display for ApiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "\t\tGithub API returned an error!\n\
        \t\tMessage: {}\n\
        \t\tURL for the Documentation: {}", self.message, self.documentation_url)
    }
}


static GITHUB_API_URL: &str = "https://api.github.com/licenses";

/// Sets the headers for the HTTP request builder.
///
/// # Arguments
///
/// * `req` - The HTTP request builder.
/// * `program_settings` - The program settings.
///
/// # Returns
///
/// The modified HTTP request builder with the headers set.
fn set_header(req: RequestBuilder, program_settings: &ProgramSettings) -> RequestBuilder {
    let mut headers = HeaderMap::new();

    if let Some(auth) = &program_settings.github_api_token {
        headers.insert(AUTHORIZATION, format!("Bearer: {}", auth).parse().unwrap());
    }
    headers.insert(USER_AGENT, "frequency403".parse().unwrap());
    headers.insert(ACCEPT, "application/vnd.github+json".parse().unwrap());
    headers.insert("X-GitHub-Api-Version", "2022-11-28".parse().unwrap());
    req.headers(headers)
}

/// Retrieves information about all licenses from the GitHub API.
///
/// # Arguments
///
/// * `program_settings` - The program settings.
///
/// # Returns
///
/// A vector of `GithubLicense` objects retrieved from the GitHub API.
///
/// # Errors
///
/// Returns a `Box<dyn Error>` if an error occurs during the retrieval process. Possible errors include:
/// * Network errors when making HTTP requests
/// * Deserialization errors when parsing the API response
/// * API error responses with error code and message
///
pub async fn get_all_licenses(
    program_settings: &ProgramSettings,
) -> Result<Vec<GithubLicense>, Box<dyn Error>> {
    let client = reqwest::Client::new();
    let mut req = client.get(GITHUB_API_URL);
    req = set_header(req, program_settings);
    let mut full_obj: Vec<GithubLicense> = vec![];


    let request_sender = req.send().await?;
    let status = request_sender.status();
    let request_body = request_sender.text().await?;
    if status == StatusCode::OK {
        if let Ok(msg) = serde_json::from_str::<ApiError>(&request_body) {
            return Err(Box::from(msg.with_error_code(status)));
        }
        for mini in
        serde_json::from_str::<Vec<MiniGithubLicense>>(&request_body)?
        {
            let mut rq = client.get(mini.url);
            rq = set_header(rq, program_settings);
            let rqs = rq.send().await?;
            let status = rqs.status();
            if rqs.status() == StatusCode::OK {
                let full_license =
                    serde_json::from_str::<GithubLicense>(rqs.text().await?.as_str())?;
                full_obj.push(full_license);
            } else {
                if let Ok(msg) = serde_json::from_str::<ApiError>(rqs.text().await?.as_str()) {
                    return Err(Box::from(msg.with_error_code(status)));
                }
                return Err(Box::from("Did not recognize the Response Error Type."));
            }
        }
    } else {
        return Err(Box::try_from(request_body).unwrap());
    }
    Ok(full_obj)
}

/// Retrieves a README template from a given URL and replaces a specified phrase with the project title.
///
/// # Arguments
///
/// * `program_settings` - A reference to the `ProgramSettings` struct containing the program settings.
/// * `directory` - A reference to the `GitDir` struct representing the project directory.
///
/// # Returns
///
/// Returns an `Option<String>` that contains the README template with the specified phrase replaced by the project title, or `None` if an error occurred
pub async fn get_readme_template(
    program_settings: &ProgramSettings,
    directory: &GitDir,
) -> Option<String> {
    let client = reqwest::Client::new();
    let mut request_builder = client.get(program_settings.readme_template_link.as_str());
    request_builder = set_header(request_builder, program_settings);

    if let Ok(response) = request_builder.send().await {
        if let Ok(body) = response.text().await {
            Some(body.replace(
                &program_settings.replace_in_readme_phrase,
                &directory.project_title,
            ))
        } else {
            None
        }
    } else {
        None
    }
}
