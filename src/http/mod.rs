use regex::Regex;
use reqwest::Url;

mod auth;
pub mod client;
pub mod download;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub fn url_regex(re: &str, url: &str) -> Option<String> {
    let u = Url::parse(url).unwrap();
    let re = Regex::new(re).unwrap();
    re.captures(&format!("{}/", u.path().trim_end_matches('/')))
        .map(|res| res.get(1).unwrap().as_str().to_owned())
}

#[cfg(test)]
mod url_test {
    use reqwest::Url;

    #[test]
    fn query() {
        let u = format!(
            "https://api.bilibili.com/x/player/wbi/playurl?cid={}&bvid={}&fnval=16",
            1, 2
        );
        let mut url = Url::parse(&u).unwrap();
        let mut pairs = url.query_pairs_mut();
        pairs.append_pair("aid", "3");
        drop(pairs);
        println!("{}", url.as_str());
    }
}
