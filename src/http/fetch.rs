use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::http::client::Client;
use crate::http::Result;
use std::time::Duration;

use super::download::Task;
use super::url_regex;

#[derive(Debug, Serialize, Deserialize)]
pub struct VideoInfo {
    pub state: i32,
    pub bvid: String,
    pub title: String,
    pub desc: String,
    pub duration: i32,
}

pub enum VideoType {
    Bangumi,
    Video,
}

impl Client {
    /// Fetch the html content of the video playback page
    pub async fn fetch_page_info(&self, url: &str) -> Result<String> {
        println!("[Fetch] 获取播放页面信息");
        let resp = self.get(url).timeout(Duration::from_secs(3)).send().await?;
        let body = resp.text().await?;
        Ok(body)
    }

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
            Ok(info)
        } else {
            Err("无法从链接解析 BV 号".into())
        }
    }

    /// Verify login based on cookies
    pub async fn validate_login(&self) -> Result<bool> {
        let url = "https://api.bilibili.com/x/web-interface/nav";
        let resp = self.get(url).timeout(Duration::from_secs(3)).send().await?;

        // Check if the response contains a login indicator
        let body = resp.text().await?;
        let is_logged_in = body.contains("\"isLogin\":true");

        if is_logged_in {
            println!("登陆成功\n");
        } else {
            println!("未登录\n");
        }

        Ok(is_logged_in)
    }

    pub async fn get_video(&self, url: &str) -> Result<Vec<Task>> {
        let info = self.fetch_video_info(url).await?;
        Ok(vec![Task::new(url.to_string(), info.title, 1)])
    }
}

pub fn process_url(url: &str) -> VideoType {
    if url.contains("bangumi") {
        VideoType::Bangumi
    } else {
        VideoType::Video
    }
}

#[cfg(test)]
mod fetch_test {
    use crate::http::client;

    use super::*;
    use reqwest::Url;
    use std::fs::File;
    use std::io::Write;
    use std::path::Path;

    #[tokio::test]
    async fn fetch_test() {
        let client = Client::new();
        // let cookies = Some(String::from("SESSDATA=7d7d07af%2C1739087102%2Ce7725%2A82CjA2_OgC1Ss9zGRGwGo-IR5Oa0esy_j93o7eutH-vtvqGEeb5GYKkXJjWp8f1hb2D6QSVjlOenVxQkRoSTZKSXU5a2Fvb2pQMW9mZEswT2dyNGJSS1FhTFg2RU9jSjBLVUhzdjBHRURmS2dLcGs5VFA5OERERzJLMmFpRFo5RkdxVGhzWjdfcW93IIEC; Domain=.bilibili.com"));
        let url = "https://www.bilibili.com/bangumi/play/ep830937?spm_id_from=333.1007.top_right_bar_window_history.content.click&from_spmid=666.25.episode.0";
        client.add_cookies("cookies.txt");
        let body = client.fetch_page_info(url).await.expect("1");
        // println!("{body}");
        let mut file = File::create("test.html").unwrap();
        file.write_all(body.as_bytes()).unwrap();
    }

    #[test]
    fn url_test() {
        let url = "https://upos-sz-estgoss.bilivideo.com/upgcxcode/92/57/1094305792/1094305792_nb3-1-30280.m4s?e=ig8euxZM2rNcNbdlhoNvNC8BqJIzNbfqXBvEqxTEto8BTrNvN0GvT90W5JZMkX_YN0MvXg8gNEV4NC8xNEV4N03eN0B5tZlqNxTEto8BTrNvNeZVuJ10Kj_g2UB02J0mN0B5tZlqNCNEto8BTrNvNC7MTX502C8f2jmMQJ6mqF2fka1mqx6gqj0eN0B599M=&uipk=5&nbs=1&deadline=1723464725&gen=playurlv2&os=upos&oi=1964813008&trid=caad2d19bba34b5c9d34eaa24e9c5841u&mid=398839362&platform=pc&og=cos&upsig=116974f3336c0b09b59913ea08eb383b&uparams=e,uipk,nbs,deadline,gen,os,oi,trid,mid,platform,og&bvc=vod&nettype=0&orderid=1,3&buvid=9572C95D-A944-1C55-26EE-4962F985BD6E28033infoc&build=0&f=u_0_0&agrr=1&bw=11003&logo=40000000";
        let u = Url::parse(url).unwrap();
        let path = u.path();
        let p = Path::new(path);
        println!("{:?}", p.file_stem());

        let extension = p.extension().unwrap().to_str().unwrap();
        println!("{}", extension);
    }

    #[tokio::test]
    async fn test_validate_login() {
        let client = Client::new();
        client.add_cookies("cookies.txt");
        let res = client.validate_login().await.unwrap();
        assert_eq!(res, true);
    }

    #[tokio::test]
    #[should_panic]
    async fn url_parser() {
        let client = client::Client::new();
        let info = client
            .fetch_video_info("https://www.bilibili.com/video")
            .await
            .unwrap();
        println!("{:#?}", info);
    }
}
