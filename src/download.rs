use crate::parser::{extract_filename, MediaInfo};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use reqwest::{header::HeaderMap, Client};
use std::process::Command;
use std::{error::Error, fs, path::PathBuf};
use tokio::{self, fs::File, io::AsyncWriteExt};

type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

#[derive(Debug)]
pub struct DownloadTask {
    /// The path to save the video
    dir: PathBuf,
    video: MediaInfo,
    audio: MediaInfo,
    client: Client,
    progress: MultiProgress,
    pub video_path: PathBuf,
    pub audio_path: PathBuf,
}

impl DownloadTask {
    pub fn new(video: MediaInfo, audio: MediaInfo, dir: Option<String>, header: HeaderMap) -> Self {
        let client = reqwest::Client::builder()
            .default_headers(header)
            .build()
            .unwrap();

        let pbm = MultiProgress::new();

        let directory: PathBuf;
        if let Some(d) = dir {
            directory = PathBuf::from(d);
        } else {
            directory = dirs::home_dir()
                .expect("failed to get home dir")
                .join("Downloads");
        }

        Self {
            dir: directory.clone(),
            video_path: directory.clone().join(Self::get_file_path(&video, "video")),
            audio_path: directory.join(Self::get_file_path(&audio, "audio")),
            video,
            audio,
            client,
            progress: pbm,
        }
    }

    async fn download_video(&self) -> Result<()> {
        let resp = self.client.get(&self.video.base_url).send().await?;

        self.download_with_bar(resp, "video").await?;
        Ok(())
    }

    async fn download_audio(&self) -> Result<()> {
        let resp = self.client.get(&self.audio.base_url).send().await?;

        self.download_with_bar(resp, "audio").await?;
        Ok(())
    }

    async fn download_with_bar(
        &self,
        mut resp: reqwest::Response,
        file_type: &'static str,
    ) -> Result<()> {
        let status = resp.status();
        if status != reqwest::StatusCode::OK {
            return Err(format!("download failed with status: {}", status).into());
        }

        let mut file: File;
        if file_type == "audio" {
            file = File::create(&self.audio_path).await?;
        } else {
            file = File::create(&self.video_path).await?;
        }

        let total_size = resp
            .content_length()
            .ok_or("Failed to get content length")?;
        let pb = self.progress.add(ProgressBar::new(total_size));
        pb.set_message(file_type);
        pb.set_style(
            ProgressStyle::with_template(r#"{spinner:.green} [{msg}] [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})"#)
            .unwrap()
            .progress_chars("#>-")
        );
        while let Some(chunk) = resp.chunk().await? {
            file.write_all(&chunk).await?;
            pb.inc(chunk.len().try_into().unwrap());
        }

        pb.finish_with_message(format!("Finished downloading {}", file_type));
        Ok(())
    }

    pub async fn execute(self, ffmpeg: Option<String>) {
        if !self.dir.exists() {
            println!("Creating directory: {}", self.dir.display());
            fs::create_dir_all(&self.dir).expect("failed to create dir");
        } else if !self.dir.is_dir() {
            panic!("The path is not a directory");
        };

        let video_dl = self.download_video();
        let audio_dl = self.download_audio();

        let res = tokio::try_join!(video_dl, audio_dl);

        match res {
            Ok((_, _)) => {
                let va_path = &self.video_path.with_extension("mp4");
                let mut cmd = if let Some(f) = ffmpeg {
                    Command::new(f)
                } else {
                    Command::new("ffmpeg")
                };
                // merge audio and video
                let merge_status = cmd
                    .arg("-i")
                    .arg(&self.video_path)
                    .arg("-i")
                    .arg(&self.audio_path)
                    .arg("-c")
                    .arg("copy")
                    .arg(va_path)
                    .status();

                match merge_status {
                    Ok(s) => {
                        if s.success() {
                            println!("Successed merged video and audio: {}", va_path.display());
                        } else {
                            eprintln!("Failed to merge video and audio");
                        }
                    }
                    Err(e) => {
                        self.remove_all();
                        panic!("Failed to merge video and audio: {}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("Download failed: {}", e);
            }
        }

        // Delete the source video and audio files
        self.remove_all();
    }

    fn get_file_path(media: &MediaInfo, file: &str) -> PathBuf {
        let filename = extract_filename(&media.base_url, file);
        PathBuf::from(filename)
    }

    pub fn remove_tmp_file(file_path: &PathBuf) {
        if file_path.exists() {
            if let Err(e) = fs::remove_file(file_path) {
                eprintln!("Failed to delete file: {}", e);
            }
        }
    }

    fn remove_all(&self) {
        Self::remove_tmp_file(&self.video_path);
        Self::remove_tmp_file(&self.audio_path);
    }
}

#[cfg(test)]
mod dl_test {
    use super::*;
    use crate::fetch::{fetch_url, init_default_header};
    use crate::parser::{choose_audio_stream, choose_video_stream, extract_play_info};
    use dirs;

    #[tokio::test]
    async fn dl() {
        let url = "https://www.bilibili.com/video/BV1ub421J7vH";
        let header = init_default_header(url, None);
        let body = fetch_url(url, None).await.expect("1");
        let mut play_info = extract_play_info(body).expect("failed to extract play info");
        let audio_stream = choose_audio_stream(&mut play_info);
        let video_stream = choose_video_stream(&mut play_info.dash.video);

        let dl = DownloadTask::new(video_stream.unwrap(), audio_stream.unwrap(), None, header);
        // println!("{:#?}", dl);
        dl.execute(None).await;
    }

    #[test]
    fn home_dir() {
        let home = dirs::home_dir().unwrap();
        println!("{}", home.display());
    }

    #[test]
    fn cmd() {
        let s = Command::new("ls")
            .arg("-l")
            .status()
            .expect("failed to execute process");

        if s.success() {
            println!("success");
        } else {
            eprintln!("failed");
        }
    }
}
