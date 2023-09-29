use std::process::exit;
use reqwest::RequestBuilder;
use crate::github_license::{GithubLicense, MiniGithubLicense};
use crate::read_input;

fn set_header(req: RequestBuilder) -> RequestBuilder {
    req.header("User-Agent", "frequency403")
        .header("User-Agent", "frequency403")
        .header("User-Agent", "frequency403")
}

pub async fn communicate() -> Option<GithubLicense> {
    let client = reqwest::Client::new();
    let mut req = client.get("https://api.github.com/licenses");
    req = set_header(req);
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

        req = set_header(req);
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