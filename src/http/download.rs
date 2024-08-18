use crate::ffmpeg::merge;
use crate::parser::{
    choose_audio_stream, choose_video_stream, extract_filename, extract_play_info, MediaInfo,
};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use reqwest::header::REFERER;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::{error::Error, fs, path::PathBuf};
use tokio::{self, fs::File, io::AsyncWriteExt};

use super::client::Client;
// use super::Result;

type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

#[derive(Debug)]
pub struct InputPath {
    pub v_path: PathBuf,
    pub a_path: PathBuf,
}

#[derive(Debug)]
pub struct Task {
    pub link: String,
    pub title: String,
    pub input_path: Mutex<Option<InputPath>>,
    pub id: i32,
    progress: MultiProgress,
}

#[derive(Debug)]
pub struct DownloadTask {
    /// The path to save the video
    pub dir: PathBuf,
    pub client: Client,
    pub tasks: Vec<Task>,
}

impl Task {
    pub fn new(link: String, title: String, id: i32) -> Self {
        Self {
            link,
            title,
            input_path: Mutex::new(None),
            id,
            progress: MultiProgress::new(),
        }
    }

    fn set_input_path(&self, v_path: PathBuf, a_path: PathBuf) {
        let mut input_path = self.input_path.lock().unwrap();
        *input_path = Some(InputPath { v_path, a_path });
    }

    fn get_media_path(&self, media: &str) -> PathBuf {
        let input_path = self.input_path.lock().unwrap();
        if media == "video" {
            input_path.as_ref().unwrap().v_path.clone()
        } else {
            input_path.as_ref().unwrap().a_path.clone()
        }
    }

    pub fn remove_media_file(&self) {
        let input_path = self.input_path.lock().unwrap();
        let v_path = &input_path.as_ref().unwrap().v_path;
        let a_path = &input_path.as_ref().unwrap().a_path;
        if v_path.exists() {
            if let Err(e) = fs::remove_file(v_path) {
                eprintln!("Failed to delete file: {}", e);
            }
        }
        if a_path.exists() {
            if let Err(e) = fs::remove_file(a_path) {
                eprintln!("Failed to delete file: {}", e);
            }
        }
    }

    async fn download(&self, client: &Client, url: &str, media: &str) -> Result<()> {
        let resp = client.get(url).header(REFERER, &self.link).send().await?;
        self.write_chunk(resp, media).await?;
        Ok(())
    }

    async fn write_chunk(&self, mut resp: reqwest::Response, media: &str) -> Result<()> {
        let status = resp.status();
        if status != reqwest::StatusCode::OK {
            return Err(format!("download failed with status: {}", status).into());
        }

        let mut file = File::create(self.get_media_path(media)).await?;

        let total_size = resp
            .content_length()
            .ok_or("Failed to get content length")?;
        let pb = self.progress.add(ProgressBar::new(total_size));
        pb.set_message(format!("downloading {media}"));
        pb.set_style(
            ProgressStyle::with_template(r#"{spinner:.green} [{msg}] [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})"#)
            .unwrap()
            .progress_chars("#>-")
        );
        while let Some(chunk) = resp.chunk().await? {
            file.write_all(&chunk).await?;
            pb.inc(chunk.len().try_into().unwrap());
        }

        pb.finish_with_message("✓");
        Ok(())
    }
}

impl DownloadTask {
    pub fn new(dir: PathBuf, client: Client, tasks: Vec<Task>) -> Self {
        Self { dir, client, tasks }
    }

    pub async fn execute(self: Arc<Self>) {
        self.create_dir_all();
        for task in self.tasks.iter() {
            println!("[Download] 下载视频: {}", task.title);
            let page_info = self
                .client
                .fetch_page_info(&task.link)
                .await
                .unwrap_or_else(|_| panic!("获取视频: {} 页面信息失败", task.title));
            let mut play_info = extract_play_info(page_info).expect("解析播放信息失败");
            let audio_stream = choose_audio_stream(&mut play_info).unwrap();
            let video_stream = choose_video_stream(&mut play_info.dash.video).unwrap();

            task.set_input_path(
                get_file_path(&self.dir, &video_stream, &format!("video{:02}", task.id)),
                get_file_path(&self.dir, &audio_stream, &format!("audio{:02}", task.id)),
            );

            let v_part = task.download(&self.client, &video_stream.base_url, "video");
            let a_part = task.download(&self.client, &audio_stream.base_url, "audio");

            let res = tokio::try_join!(v_part, a_part);
            match res {
                Ok((_, _)) => {
                    let o_path = self.dir.join(&task.title).with_extension("mp4");
                    // merge audio and video
                    let merge_status = merge(
                        task.get_media_path("audio"),
                        task.get_media_path("video"),
                        &o_path,
                    );

                    match merge_status {
                        Ok(_) => println!("下载完成: {}\n", o_path.display()),
                        Err(e) => {
                            panic!("Failed to merge video and audio: {}", e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("下载失败: {}", e);
                }
            }
        }
    }

    pub fn remove_tmp_file(&self) {
        for task in self.tasks.iter() {
            task.remove_media_file();
        }
    }

    fn create_dir_all(&self) {
        fs::create_dir_all(&self.dir).expect("下载文件夹新建失败");
        println!("下载路径: {}\t", self.dir.display());
    }
}

fn get_file_path(dir: &Path, media: &MediaInfo, default: &str) -> PathBuf {
    let filename = extract_filename(&media.base_url, default);
    dir.join(PathBuf::from(filename))
}

#[cfg(test)]
mod dl_test {
    use super::*;
    use crate::http::client;
    use dirs;

    #[tokio::test]
    async fn dl() {
        let url = "https://www.bilibili.com/video/BV1ub421J7vH";
        let client = client::Client::new();
        let v = client.get_video(url).await.unwrap();

        let dl = Arc::new(DownloadTask::new(
            dirs::home_dir().unwrap().join("Downloads"),
            client,
            v,
        ));
        println!("{:#?}", dl);
        let cdl = dl.clone();
        let _ = tokio::spawn(async move {
            cdl.execute().await;
        })
        .await;
        dl.remove_tmp_file();
    }

    #[test]
    fn home_dir() {
        let home = dirs::home_dir().unwrap();
        println!("{}", home.display());
    }
}
