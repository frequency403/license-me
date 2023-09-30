use std::process::exit;
use reqwest::header::{ACCEPT, AUTHORIZATION, USER_AGENT};
use reqwest::RequestBuilder;
use crate::github_license::{GithubLicense, MiniGithubLicense};
use crate::read_input;
use crate::settings_file::ProgramSettings;

fn set_header(req: RequestBuilder, program_settings: &ProgramSettings) -> RequestBuilder {
    let mut header_holder = req;
    if let Some(auth) = &program_settings.github_api_token{
        header_holder = req.header(AUTHORIZATION, format!("Bearer: {}", auth));
    }
    header_holder = req.header(USER_AGENT, "frequency403")
        .header(ACCEPT, "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28");
    header_holder
}

pub async fn communicate(program_settings: &ProgramSettings) -> Option<GithubLicense> {
    let client = reqwest::Client::new();
    let mut req = client.get("https://api.github.com/licenses");
    req = set_header(req, program_settings);
    let some = req.send().await.unwrap();
    if let Ok(body) = some.text().await {
        let obj: Vec<MiniGithubLicense> = serde_json::from_str::<Vec<MiniGithubLicense>>(body.as_str()).unwrap();
        obj.iter().enumerate()
            .for_each(|(c, l)| {
                println!("[{}] {}",c+1, l.name)
            }
            );
        let user_input = read_input("Your Selection: ");

        match user_input.parse::<usize>() {
            Ok(o) => {req = client.get(obj[o-1].clone().url)}
            Err(e) => {println!("{}",e); exit(1)}
        }

        req = set_header(req, program_settings);
        if let Ok(some) = req.send().await {
            if let Ok(some_other) = some.text().await {
                Some(serde_json::from_str::<GithubLicense>(some_other.as_str()).unwrap())
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    }
}

pub async fn get_readme_template(program_settings: &ProgramSettings) -> Option<String> {
    let client = reqwest::Client::new();
    let mut request_builder = client.get(program_settings.license_template_link.as_str());
    request_builder = set_header(request_builder, program_settings);

    if let Ok(response) = request_builder.send().await {
        if let Ok(body) = response.text().await {
            Some(body)
        } else { None }
    } else { None }
}