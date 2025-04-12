use reqwest::Url;
use serde::{self, Deserialize, Serialize};
use serde_json::Value;
use std::path::Path;

use crate::client::Client;
use crate::debug;
use crate::utils::Result;
use std::time::Duration;

use crate::download::{Params, Task};
use crate::stream::{Stream, Streams};
use crate::utils::url_regex;

fn i64_to_string<'de, D>(deserializer: D) -> std::result::Result<String, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    let i64_value = i64::deserialize(deserializer)?;
    Ok(i64_value.to_string())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VideoInfo {
    #[serde(deserialize_with = "i64_to_string")]
    pub cid: String,
    pub state: i32,
    pub bvid: String,
    pub title: String,
    pub desc: String,
    pub duration: i32,
    /// part number (default: 1)
    pub videos: i32,
    /// start from 1
    pub pages: Vec<Page>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Page {
    #[serde(deserialize_with = "i64_to_string")]
    pub cid: String,
    /// start from 1
    pub page: i32,
    #[serde(rename = "part")]
    pub page_title: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Dolby {
    #[serde(rename = "type")]
    type_: i32,
    pub audio: Option<Vec<Stream>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Flac {
    display: bool,
    pub audio: Option<Stream>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Dash {
    pub audio: Vec<Stream>,
    pub video: Vec<Stream>,
    pub dolby: Dolby,
    pub flac: Option<Flac>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DashResult {
    accept_description: Vec<String>,
    accept_format: String,
    pub dash: Dash,
}

pub enum VideoType {
    Bangumi,
    Video,
}

pub fn process_url(url: &str) -> VideoType {
    if url.contains("bangumi") {
        VideoType::Bangumi
    } else {
        VideoType::Video
    }
}

impl Client {
    /// Fetch video information via BV code
    async fn fetch_video_info(&self, url: &str) -> Result<VideoInfo> {
        if let Some(code) = url_regex(r"/BV(\S+)/", url) {
            let mut resp: Value = self
                .get(&format!(
                    "https://api.bilibili.com/x/web-interface/view?bvid=BV{code}"
                ))
                .timeout(Duration::from_secs(3))
                .send()
                .await?
                .json()
                .await?;
            let info: VideoInfo = serde_json::from_value(resp["data"].take())?;
            debug!("video info: {info:#?}");
            Ok(info)
        } else {
            Err("无法从链接解析 BV 号".into())
        }
    }

    pub async fn get_video(&self, url: &str) -> Result<(String, Vec<Task>)> {
        let info = self.fetch_video_info(url).await?;
        if info.videos == 1 {
            let params = vec![("bvid".into(), info.bvid), ("cid".into(), info.cid)];
            Ok((
                format!("视频: [{}]", info.title),
                vec![Task::new(
                    1,
                    "https://api.bilibili.com/x/player/wbi/playurl".into(),
                    params,
                    info.title,
                )],
            ))
        } else {
            let mut page_list = vec![];
            for (i, page) in info.pages.into_iter().enumerate() {
                let params = vec![("bvid".into(), info.bvid.clone()), ("cid".into(), page.cid)];
                let title = format!("{}p: {}", i + 1, page.page_title);
                let task = Task::new(
                    i + 1,
                    "https://api.bilibili.com/x/player/wbi/playurl".into(),
                    params,
                    title,
                );
                page_list.push(task);
            }
            Ok((
                format!("分p视频: [{}] (共{}p)", info.title, info.videos),
                page_list,
            ))
        }
    }

    /// 获取流
    pub async fn fetch_dash(&self, url: &str, parmas: &Params) -> Result<Streams> {
        let u = self.encode_url(url, parmas.clone())?;
        let mut res: Value = self.get(&u).send().await?.json().await?;
        // println!("{:#?}", res);
        let data;
        if u.contains("pgc") {
            data = res["result"].take();
        } else {
            data = res["data"].take();
        }
        let dash: DashResult = serde_json::from_value(data)?;
        let s = Streams::from(dash);
        Ok(s)
    }
}

pub fn extract_filename(url: &str, default: &str) -> String {
    let u = Url::parse(url).unwrap();
    let path = Path::new(u.path());
    let filename = path.file_stem().unwrap().to_str().unwrap_or(default);
    format!(
        "{}.{}",
        filename,
        path.extension().unwrap().to_str().unwrap()
    )
}
