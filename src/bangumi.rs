use crate::http::download::Task;
use crate::http::Result;
use crate::http::{client::Client, url_regex};
use serde::{Deserialize, Serialize};
use serde_json::{self, Value};
use std::path::PathBuf;
use BangumiID::*;

enum BangumiID {
    MediaID(i64),
    SeasonID(i64),
    EpisodeID(i64),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BangumiInfo {
    /// 动漫名称
    title: String,
    /// 播放页面链接
    link: String,
    season_id: i64,
    media_id: i64,
    episodes: Vec<Episode>,
    season_title: String,
    /// 总集数
    total: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Episode {
    ep_id: i64,
    pub long_title: String,
    pub link: String,
    #[serde(rename = "title")]
    pub ep_num: String,
    pub badge_type: i32,
    status: i32,
}

impl Client {
    /// get list of bangumi episodes via **season_id**
    async fn fetch_bangumi_info(&self, id_name: &str, id: i64) -> Result<BangumiInfo> {
        let url = format!(
            "https://api.bilibili.com/pgc/view/web/season?{}={}",
            id_name, id
        );
        let mut resp: Value = self.get(&url).send().await?.json().await?;
        let info: BangumiInfo = serde_json::from_value(resp["result"].take())?;

        Ok(info)
    }

    /// fetch bangumi **season_id** via **media_id**
    async fn fetch_bangumi_sid(&self, id: i64) -> Result<i64> {
        let url = format!("https://api.bilibili.com/pgc/review/user?media_id={id}");
        let mut resp: Value = self.get(&url).send().await?.json().await?;

        let sid = resp["result"]["media"]["season_id"]
            .take()
            .as_i64()
            .unwrap();
        Ok(sid)
    }

    pub async fn get_bangumi(&self, url: &str, dir: &mut PathBuf) -> Result<Vec<Task>> {
        let info = match bangumi_url_parser(url) {
            Ok(SeasonID(id)) => self.fetch_bangumi_info("season_id", id).await?,
            Ok(MediaID(id)) => {
                let sid = self.fetch_bangumi_sid(id).await?;
                self.fetch_bangumi_info("season_id", sid).await?
            }
            Ok(EpisodeID(id)) => {
                let info = self.fetch_bangumi_info("ep_id", id).await?;
                let ep_list = info.episodes;
                let target = ep_list.into_iter().find(|ep| ep.ep_id == id);
                if let Some(ep) = target {
                    return Ok(vec![Task::new(
                        ep.link,
                        get_bangumi_file_name(&info.title, &ep.ep_num, &ep.long_title),
                        ep.ep_num.parse().unwrap(),
                    )]);
                }
                return Err("未找到番剧".into());
            }
            Err(e) => {
                return Err(e);
            }
        };
        println!("获取番剧列表成功\n《{}》, 共{}集", &info.title, info.total);
        dir.push(&info.title);
        let mut video_list: Vec<Task> = Vec::new();
        let filtered_ep_list = info.episodes.into_iter().filter(|ep| ep.badge_type != 1);

        // println!("{:#?}", filtered_ep_list);
        filtered_ep_list.enumerate().for_each(|(i, ep)| {
            video_list.push(Task::new(
                ep.link,
                get_bangumi_file_name(&info.title, &ep.ep_num, &ep.long_title),
                i,
            ))
        });
        Ok(video_list)
    }
}

/// 从链接番剧匹配 season_id 或 media_id
fn bangumi_url_parser(url: &str) -> Result<BangumiID> {
    if let Some(id) = url_regex(r"/md(\d+)/", url) {
        let id: i64 = id.parse()?;
        return Ok(MediaID(id));
    }
    if let Some(id) = url_regex(r"/ss(\d+)/", url) {
        let id: i64 = id.parse()?;
        return Ok(SeasonID(id));
    }
    if let Some(id) = url_regex(r"/ep(\d+)/", url) {
        let id: i64 = id.parse()?;
        return Ok(EpisodeID(id));
    }
    Err("解析番剧 id 失败".into())
}

fn get_bangumi_file_name(b_title: &str, ep_num: &str, ep_title: &str) -> String {
    let formatted_ep_num = if let Ok(num) = ep_num.parse::<i32>() {
        format!("{:02}", num)
    } else {
        ep_num.to_string()
    };
    format!("{} - {} [{}]", b_title, formatted_ep_num, ep_title)
}

#[cfg(test)]
mod bangumi {

    use crate::http::client::Client;

    #[tokio::test]
    async fn test_bangumi_eps() {
        let client = super::Client::new();
        let id = 6038;
        let b_info = client.fetch_bangumi_info("season_id", id).await.unwrap();
        println!("{:#?}", b_info);
    }

    #[tokio::test]
    async fn test_bangumi_info() {
        let client = super::Client::new();
        let id = 21231728;
        let b_info = client.fetch_bangumi_sid(id).await.unwrap();
        println!("{:#?}", b_info);
    }

    #[tokio::test]
    async fn get_bangumi() {
        let client = Client::new();
        // let url = "https://www.bilibili.com/bangumi/play/ss47561?spm_id_from=333.999.0.0";
        let url = "https://www.bilibili.com/bangumi/media/md21231728";
        let mut dir = dirs::home_dir().unwrap();
        let info = client.get_bangumi(url, &mut dir).await.unwrap();
        println!("{:#?}", info);
    }
}
