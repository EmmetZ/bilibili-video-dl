use reqwest::{
    cookie::Jar,
    header::{HeaderMap, USER_AGENT},
    Url,
};

use std::{fs, sync::Arc};

const UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/127.0.0.0 Safari/537.36 Edg/127.0.0.0";

#[derive(Debug)]
pub struct Client {
    cli: reqwest::Client,
    cookies: Arc<Jar>,
}

impl Client {
    pub fn new() -> Self {
        let cookies = Arc::new(Jar::default());
        let mut header = HeaderMap::new();
        header.insert(USER_AGENT, UA.parse().unwrap());

        let cli = reqwest::Client::builder()
            .cookie_provider(cookies.clone())
            .default_headers(header)
            .build()
            .unwrap();
        Self { cli, cookies }
    }

    pub fn add_cookies(&self, c_path: &str) {
        let s = fs::read_to_string(c_path).expect("failed to read cookies file");
        let lines: Vec<&str> = s.lines().collect();
        for cookie in lines {
            let mut cookie_parts = cookie.split(';');
            let c = cookie_parts.next().unwrap();
            let domain = cookie_parts.next().unwrap().trim();
            self.cookies.add_cookie_str(
                &format!("{}; Domain={}", c, domain),
                &Url::parse("https://www.bilibili.com").unwrap(),
            );
        }
    }

    pub fn get(&self, url: &str) -> reqwest::RequestBuilder {
        self.cli.get(url)
    }
}

#[cfg(test)]
mod client {
    use super::*;

    #[tokio::test]
    async fn test_client() {
        let client = Client::new();

        let c = client
            .cli
            .get("https://www.baidu.com")
            .header(USER_AGENT, UA)
            .send()
            .await
            .unwrap();

        // println!("{:#?}", client);
        println!("{:#?}", c.headers());
        // println!("{:#?}", client.cookies);
    }

    #[test]
    fn test_encode_cookies() {
        let client = Client::new();
        client.add_cookies("cookies.txt");
        println!("{:#?}", client.cookies);
    }
}
