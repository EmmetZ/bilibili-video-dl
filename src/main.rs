mod cli;
mod download;
mod fetch;
mod parser;

use clap::Parser;
use cli::Cli;
use download::DownloadTask;
use fetch::{encode_cookies, fetch_url, init_default_header};
use parser::{choose_audio_stream, choose_video_stream, extract_play_info};
use std::sync::Arc;

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
    let dl = Arc::new(DownloadTask::new(
        video_stream.unwrap(),
        audio_stream.unwrap(),
        cli.dl_dir,
        header,
    ));
    // println!("{:#?}", dl);

    let listen_task = tokio::spawn(listen_for_interrupt());

    let clone_dl = Arc::clone(&dl);
    let download_task = tokio::spawn(async move {
        clone_dl.execute(cli.ffmpeg).await;
    });

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

    // remove tmp files
    dl.remove_all();
}

async fn listen_for_interrupt() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to listen for ctrl-c event");
}
