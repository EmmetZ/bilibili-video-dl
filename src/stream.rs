use crate::http::Result;
use serde::{self, Deserialize, Serialize};

use crate::video::DashResult;

pub const VIDEO_QUALITY: [(i32, &str); 9] = [
    (6, "240P 极速"),
    (16, "360P 流畅"),
    (32, "480P 清晰"),
    (64, "720P 高清"),
    (74, "720P60 高帧率"),
    (80, "1080P 高清"),
    (112, "1080P+ 高码率"),
    (116, "1080P60 高帧率"),
    (120, "4K 超清"),
];

#[derive(Serialize, Deserialize, Debug)]
struct SegmentBase {
    initialization: String,
    index_range: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Streams {
    list: Vec<Stream>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Stream {
    pub id: i32,
    pub base_url: String,
    backup_url: Vec<String>,
    bandwidth: i32,
    codecs: String,
    mime_type: String,
    width: i32,
    height: i32,
    segment_base: SegmentBase,
}

pub enum MediaType {
    Video,
    Audio,
}

// type MediaInfoOption = Option<Stream>;

impl Streams {
    fn new() -> Self {
        Streams { list: Vec::new() }
    }
}

impl From<DashResult> for Streams {
    fn from(mut value: DashResult) -> Self {
        let mut ss = Self::new();
        ss.list.append(&mut value.dash.video);
        ss.list.append(&mut value.dash.audio);

        let dolby = value.dash.dolby.audio;
        if dolby.is_some() {
            ss.list.append(&mut dolby.unwrap());
        }

        let flac = value.dash.flac;
        if flac.is_some() {
            let f_audio = flac.unwrap().audio;
            if let Some(a) = f_audio {
                ss.list.push(a);
            }
        }
        ss
    }
}

impl Streams {
    pub fn best(&self, media_type: MediaType) -> Result<&Stream> {
        if self.list.len() == 0 {
            Err("Stream list is empty".into())
        } else {
            let pat = match media_type {
                MediaType::Video => "video",
                MediaType::Audio => "audio",
            };
            let s = self
                .list
                .iter()
                .filter(|m| m.mime_type.contains(pat))
                .max_by_key(|f| f.bandwidth);
            Ok(s.unwrap())
        }
    }
}
