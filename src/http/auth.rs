use crate::http::client::Client;
use crate::http::Result;
use reqwest::Url;
use serde::{self, Deserialize, Serialize};
use serde_json::Value;
use std::char;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use super::download::Params;

const MIXIN_KEY_ENC_TAB: [u8; 64] = [
    46, 47, 18, 2, 53, 8, 23, 32, 15, 50, 10, 31, 58, 3, 45, 35, 27, 43, 5, 49, 33, 9, 42, 19, 29,
    28, 14, 39, 12, 38, 41, 13, 37, 48, 7, 16, 24, 55, 40, 61, 26, 17, 0, 1, 60, 51, 30, 4, 22, 25,
    54, 21, 56, 59, 6, 63, 57, 62, 11, 36, 20, 34, 44, 52,
];

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct UserStatus {
    #[serde(rename = "isLogin")]
    pub is_login: bool,
    #[serde(default)]
    pub uname: String,
    #[serde(rename = "wbi_img")]
    pub wbi: Wbi,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Wbi {
    #[serde(deserialize_with = "key_parser", rename = "img_url")]
    img_key: String,
    #[serde(deserialize_with = "key_parser", rename = "sub_url")]
    sub_key: String,
    #[serde(default)]
    mixin_key: String,
    // #[serde(default)]
    // wts: String,
}

fn key_parser<'de, D>(deserializer: D) -> std::result::Result<String, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    let url =
        Url::parse(&String::deserialize(deserializer)?).expect("error, img_url/sub_url is not url");
    let raw_key_with_ext = url
        .path_segments()
        .expect("error, invalid img_url/sub_url")
        .last();
    if let Some(r) = raw_key_with_ext {
        let (raw_key, _) = r.split_once('.').unwrap();
        Ok(raw_key.to_string())
    } else {
        panic!("error, img_url/sub_url does not exist");
    }
}

impl Client {
    pub async fn fetch_user_status(&self) -> Result<UserStatus> {
        let url = "https://api.bilibili.com/x/web-interface/nav";
        let mut resp: Value = self
            .get(url)
            .timeout(Duration::from_secs(3))
            .send()
            .await?
            .json()
            .await?;
        let mut user: UserStatus = serde_json::from_value(resp["data"].take())?;
        user.wbi.mixin_key = user.wbi.gen_mixin_key();
        Ok(user)
    }

    pub fn encode_url(&self, url: &str, mut parmas: Params) -> Result<String> {
        let timestamp = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(t) => t.as_secs(),
            Err(_) => panic!("SystemTime before UNIX EPOCH!"),
        };
        parmas.push(("wts".into(), timestamp.to_string()));
        // 视频流web URL, dash格式 `fnval=16`
        parmas.push(("fnval".into(), "16".into()));
        parmas.sort_by(|a, b| a.0.cmp(&b.0));

        let query = parmas
            .iter()
            .map(|(pn, pv)| format!("{}={}", pn, pv))
            .collect::<Vec<_>>()
            .join("&");

        let wbi_sign = format!("{:?}", md5::compute(query.clone() + self.get_mixin_key()?));
        let encoded_query = query + &format!("&w_rid={}", wbi_sign);
        let mut u = Url::parse(url)?;
        u.set_query(Some(&encoded_query));
        Ok(u.to_string())
    }
}

/// wbi ref: https://socialsisteryi.github.io/bilibili-API-collect/docs/misc/sign/wbi.html#wbi-%E7%AD%BE%E5%90%8D%E7%AE%97%E6%B3%95
impl Wbi {
    pub fn gen_mixin_key(&self) -> String {
        let raw = format!("{}{}", &self.img_key, &self.sub_key);
        let raw_key: &[u8] = raw.as_ref();
        MIXIN_KEY_ENC_TAB
            .iter()
            .take(32)
            .map(|n| raw_key[*n as usize] as char)
            .collect::<String>()
    }

    pub fn get_mixin_key(&self) -> Result<&str> {
        if self.mixin_key.len() > 0 {
            Ok(&self.mixin_key)
        } else {
            Err("mixin_key is empty".into())
        }
    }
}

#[cfg(test)]
mod user_test {
    use crate::http::client::Client;

    #[tokio::test]
    async fn get_status() {
        let client = Client::new();
        let user = client.fetch_user_status().await.unwrap();
        assert_eq!("ea1db124af3c7062474693fa704f4ff8", user.wbi.gen_mixin_key());
    }
}
