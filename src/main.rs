mod cli;
mod download;
mod fetch;
mod parser;

use std::path::PathBuf;

use clap::Parser;
use cli::Cli;
use download::DownloadTask;
use fetch::{encode_cookies, fetch_url, init_default_header};
use parser::{choose_audio_stream, choose_video_stream, extract_play_info};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let cookies: Option<String> = match cli.cookies {
        Some(c) => encode_cookies(c).unwrap(),
        None => None,
    };

    let url = cli.url.as_str();
    let body = fetch_url(url, cookies.as_ref())
        .await
        .expect("failed to fetch video page");
    let mut play_info = extract_play_info(body).expect("failed to extract play info");
    let audio_stream = choose_audio_stream(&mut play_info);
    let video_stream = choose_video_stream(&mut play_info.dash.video);

    let header = init_default_header(url, cookies.as_ref());
    let dl = DownloadTask::new(
        video_stream.unwrap(),
        audio_stream.unwrap(),
        cli.dl_dir,
        header,
    );
    // println!("{:#?}", dl);

    let listen_task = tokio::spawn(listen_for_interrupt(
        dl.video_path.clone(),
        dl.audio_path.clone(),
    ));

    let download_task = tokio::spawn(dl.execute(cli.ffmpeg));

    tokio::select! {
        _ = listen_task => {
            println!("task canceled by user");
        }
        res = download_task => {
            match res {
                Ok(_) => {
                    println!("Completed");
                },
                Err(e) => {
                    eprintln!("panicked: {:?}", e)
                }
            }
        }
    }
}

async fn listen_for_interrupt(video_path: PathBuf, audio_path: PathBuf) {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to listen for ctrl-c event");
    DownloadTask::remove_tmp_file(&video_path);
    DownloadTask::remove_tmp_file(&audio_path);
}
