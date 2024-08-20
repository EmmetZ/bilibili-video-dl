mod bangumi;
mod cli;
mod ffmpeg;
mod http;
mod parser;
mod tui;

use clap::Parser;
use cli::Cli;
use http::{
    client::Client,
    download::DownloadTask,
    fetch::{process_url, VideoType},
};
use std::sync::Arc;
use tui::{select_download_video, wait, SelectionUI};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let mut dir = cli.dl_dir;
    let client = Client::new();

    if let Some(c) = cli.cookies {
        println!("添加 cookies");
        client.add_cookies(&c);
    };

    let _ = client.validate_login().await;

    let url = cli.url.as_str();
    let video_list = match process_url(url) {
        VideoType::Bangumi => client
            .get_bangumi(url, &mut dir)
            .await
            .expect("获取番剧列表失败"),
        VideoType::Video => client.get_video(url).await.expect("获取视频失败"),
    };

    wait();
    let mut sui = SelectionUI::new(&video_list);
    sui.run().expect("Failed to run tui");
    let res = sui.get_selection();
    let selected_video_list = select_download_video(video_list, res);

    if selected_video_list.is_empty() {
        return;
    }

    let dl = Arc::new(DownloadTask::new(dir, client, selected_video_list));
    let listen_task = tokio::spawn(listen_for_interrupt());

    let clone_dl = Arc::clone(&dl);
    let download_task = tokio::spawn(async move {
        clone_dl.execute().await;
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
    dl.remove_tmp_file();
}

async fn listen_for_interrupt() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to listen for ctrl-c event");
}
