use reqwest::header::{HeaderMap, ACCEPT, AUTHORIZATION, USER_AGENT};
use reqwest::RequestBuilder;
use std::error::Error;
use crate::git_dir::GitDir;
use crate::github_license::{GithubLicense, MiniGithubLicense};
use crate::settings_file::ProgramSettings;

static GITHUB_API_URL: &str = "https://api.github.com/licenses";

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

pub async fn get_all_licenses(
    program_settings: &ProgramSettings,
) -> Result<Vec<GithubLicense>, Box<dyn Error>> {
    let client = reqwest::Client::new();
    let mut req = client.get(GITHUB_API_URL);
    req = set_header(req, program_settings);
    let mut full_obj: Vec<GithubLicense> = vec![];
    for mini in
        serde_json::from_str::<Vec<MiniGithubLicense>>(req.send().await?.text().await?.as_str())?
    {
        let mut rq = client.get(mini.url);
        rq = set_header(rq, program_settings);
        let full_license =
            serde_json::from_str::<GithubLicense>(rq.send().await?.text().await?.as_str())?;
        full_obj.push(full_license);
    }
    Ok(full_obj)
}

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
