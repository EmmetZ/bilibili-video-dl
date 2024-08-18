use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_json::{self, Value};
use std::{error::Error, path::Path};

#[derive(Serialize, Deserialize, Debug)]
struct SegmentBase {
    initialization: String,
    index_range: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MediaInfo {
    id: i32,
    pub base_url: String,
    backup_url: Vec<String>,
    bandwidth: i32,
    codecs: String,
    mime_type: String,
    width: i32,
    height: i32,
    segment_base: SegmentBase,
}

type MediaInfoOption = Option<MediaInfo>;

#[derive(Serialize, Deserialize, Debug)]
struct Dolby {
    #[serde(rename = "type")]
    type_: i32,
    audio: Option<Vec<MediaInfoOption>>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Flac {
    display: bool,
    audio: Option<MediaInfoOption>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Dash {
    audio: Vec<MediaInfoOption>,
    pub video: Vec<MediaInfoOption>,
    dolby: Dolby,
    flac: Option<Flac>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PlayInfo {
    accept_description: Vec<String>,
    accept_format: String,
    pub dash: Dash,
}

fn find_start_token<'a>(
    body: &str,
    start_tokens: &[&str],
    end_tokens: &'a [&str],
) -> (usize, Option<usize>, &'a str) {
    for (i, token) in start_tokens.iter().enumerate() {
        if let Some(index) = body.find(token) {
            return (i, Some(index), end_tokens[i]);
        }
    }
    (0, None, "")
}

pub fn extract_play_info(body: String) -> Result<PlayInfo, Box<dyn Error>> {
    println!("[Parsing] 解析视频链接......");
    let start_tokens = ["<script>window.__playinfo__=", "const playurlSSRData = "];
    let end_tokens = ["</script>", "if"];

    let (index, start, end_token) = find_start_token(&body, &start_tokens, &end_tokens);

    if start.is_none() {
        return Err("failed to find play info".into());
    }

    let start_index = start.unwrap() + start_tokens[index].len();
    let end_index = body[start_index..].find(end_token).unwrap() + start_index;
    let mut serde_res = serde_json::from_str::<Value>(&body[start_index..end_index])?;
    // print!("{:#?}", serde_res);
    if index == 0 {
        let play_info: PlayInfo = serde_json::from_value(serde_res["data"].take())?;
        // println!("{:#?}", play_info);
        return Ok(play_info);
    } else if index == 1 {
        let play_info: PlayInfo = serde_json::from_value(serde_res["result"]["video_info"].take())?;
        // println!("{:#?}", play_info);
        return Ok(play_info);
    }
    Err("failed to extract play info".into())
}

pub fn choose_audio_stream(play_info: &mut PlayInfo) -> MediaInfoOption {
    let mut audio_data = Vec::new();
    for audio in &mut play_info.dash.audio {
        audio_data.push(audio);
    }

    let dolby = play_info.dash.dolby.audio.as_mut();
    if dolby.is_some() {
        let dolby_audio = dolby.unwrap();
        for audio in dolby_audio {
            audio_data.push(audio);
        }
    }

    let flac = play_info.dash.flac.as_mut();
    if flac.is_some() {
        if let Some(f) = flac {
            let a = &mut f.audio;
            a.is_some().then(|| audio_data.push(a.as_mut().unwrap()));
        }
    }

    audio_data
        .into_iter()
        .max_by_key(|f| f.as_ref().unwrap().bandwidth)
        .unwrap()
        .take()
}

pub fn choose_video_stream(video_data: &mut [MediaInfoOption]) -> MediaInfoOption {
    video_data
        .iter_mut()
        .max_by_key(|f| f.as_ref().unwrap().bandwidth)
        .unwrap()
        .take()
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

#[cfg(test)]
mod parser_test {
    use crate::http::client;

    use super::*;

    #[tokio::test]
    async fn parser_test() {
        let client = client::Client::new();
        // let url = "https://www.bilibili.com/video/BV1ub421J7vH";
        let url = "https://www.bilibili.com/video/BV1P1421t75S/?spm_id_from=333.337.search-card.all.click&vd_source=7b61f7ca2c7edcd57c0ffd1c17ee4e4c";
        let body = client.fetch_page_info(url).await.expect("1");

        let mut play_info = extract_play_info(body).expect("failed to extract play info");
        // println!("{:#?}", play_info);
        let audio_stream = choose_audio_stream(&mut play_info);

        let video_stream = choose_video_stream(&mut play_info.dash.video);
        println!("{:#?}", video_stream);
        println!("{:#?}", audio_stream);
    }
}
