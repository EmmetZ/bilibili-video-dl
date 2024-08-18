use regex::Regex;
use reqwest::Url;

pub mod client;
pub mod download;
pub mod fetch;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub fn url_regex(re: &str, url: &str) -> Option<String> {
    let u = Url::parse(url).unwrap();
    let re = Regex::new(re).unwrap();
    re.captures(&format!("{}/", u.path().trim_end_matches('/')))
        .map(|res| res.get(1).unwrap().as_str().to_owned())
}
