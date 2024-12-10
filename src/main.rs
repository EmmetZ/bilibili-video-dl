mod bangumi;
mod cli;
mod ffmpeg;
mod http;
mod stream;
mod tui;
mod video;

use clap::Parser;
use cli::Cli;
use http::{client::Client, download::DownloadTask};
use std::sync::Arc;
use tui::{select_download_video, SelectionUI};

use crate::video::{process_url, VideoType};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    println!("loading...");
    let mut dir = cli.dl_dir;
    let mut client = Client::new();

    if let Some(c) = &cli.cookies {
        client.add_cookies(c);
    };

    let uname = client
        .update_user_statue()
        .await
        .expect("update user status failed");

    let url = cli.url.as_str();
    let (msg, video_list) = match process_url(url) {
        VideoType::Bangumi => client
            .get_bangumi(url, &mut dir)
            .await
            .expect("获取番剧列表失败"),
        VideoType::Video => client.get_video(url).await.expect("获取视频失败"),
    };

    let mut sui = SelectionUI::new(&msg, uname.as_ref(), &video_list);
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
    dl.remove_tmp_file();
}

async fn listen_for_interrupt() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to listen for ctrl-c event");
}
