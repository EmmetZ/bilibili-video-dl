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

pub fn extract_play_info(body: String) -> Result<PlayInfo, Box<dyn Error>> {
    let start_token = "<script>window.__playinfo__=";
    let end_token = "</script>";

    let start_index = body.find(start_token).unwrap() + start_token.len();
    let end_index = body[start_index..].find(end_token).unwrap() + start_index;

    let mut serde_res = serde_json::from_str::<Value>(&body[start_index..end_index])?;
    // print!("{:#?}", serde_res);

    let play_info: PlayInfo = serde_json::from_value(serde_res["data"].take())?;
    // println!("{:#?}", play_info);

    Ok(play_info)
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
            let a = f.audio.as_mut().unwrap();
            audio_data.push(a);
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
    use super::*;
    use crate::fetch::fetch_url;

    #[tokio::test]
    async fn parser_test() {
        // let url = "https://www.bilibili.com/video/BV1ub421J7vH";
        let url = "https://www.bilibili.com/video/BV1P1421t75S/?spm_id_from=333.337.search-card.all.click&vd_source=7b61f7ca2c7edcd57c0ffd1c17ee4e4c";
        let body = fetch_url(url, None).await.expect("1");

        // let _ = extract_play_info(body);
        let mut play_info = extract_play_info(body).expect("failed to extract play info");
        let audio_stream = choose_audio_stream(&mut play_info);

        let video_stream = choose_video_stream(&mut play_info.dash.video);
        println!("{:#?}", video_stream);
        println!("{:#?}", audio_stream);
    }
}
