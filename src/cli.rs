use std::{
    io::{stdin, stdout, Stdout, Write},
    path::PathBuf,
};

use clap::Parser;
use reqwest::Url;
use termion::{
    clear, color, cursor,
    event::Key,
    input::TermRead,
    raw::{IntoRawMode, RawTerminal},
    style,
};

use crate::http::download::Task;

const SELECTED_COLOR: (u8, u8, u8) = (255, 167, 38);
const UNSELECTED_COLOR: (u8, u8, u8) = (120, 144, 156);

#[derive(Parser, Debug)]
#[command(name = "bili-dl")]
#[command(version = "1.1")]
pub struct Cli {
    /// 视频链接
    #[arg(value_parser = validate_url)]
    pub url: Url,

    /// cookies.txt 的路径
    #[arg(long, short)]
    pub cookies: Option<String>,

    /// 下载目录，默认为用户下载目录
    #[arg(long, short, value_parser = set_dir, default_value = "")]
    pub dl_dir: PathBuf,
}

fn validate_url(url: &str) -> Result<Url, String> {
    let u = Url::parse(url).expect("链接格式错误");
    Ok(u)
}

fn set_dir(dir: &str) -> Result<PathBuf, String> {
    if !dir.is_empty() {
        let d = PathBuf::from(dir);
        if !d.exists() || !d.is_dir() {
            return Err("文件夹不存在".into());
        }
        Ok(PathBuf::from(dir))
    } else {
        match std::env::current_dir() {
            Ok(d) => Ok(d),
            Err(e) => Err(e.to_string()),
        }
    }
}

pub fn select_download_video(video_list: Vec<Task>) -> Vec<Task> {
    let stdin = stdin();
    let mut buf = stdout().into_raw_mode().unwrap();

    let mut flag = true;
    let mut is_selected = vec![flag; video_list.len()];
    let mut line: usize = 0;

    termion_init(&mut buf);

    print_video_list(&mut buf, &video_list, &is_selected, line);
    buf.flush().unwrap();

    for k in stdin.keys() {
        match k.unwrap() {
            Key::Up => {
                line = line.saturating_sub(1);
            }
            Key::Down => {
                if line < video_list.len() - 1 {
                    line += 1
                }
            }
            Key::Char('d') => {
                is_selected[line] = !is_selected[line];
            }
            Key::Char('y') => {
                break;
            }
            Key::Char('n') => {
                println!("取消下载");
                is_selected = vec![false; video_list.len()];
                break;
            }
            Key::Char('a') => {
                flag = !flag;
                is_selected = vec![flag; video_list.len()];
            }
            _ => {}
        }

        print_video_list(&mut buf, &video_list, &is_selected, line);
        buf.flush().unwrap();
    }

    write!(buf, "{}", termion::cursor::Show).unwrap();
    buf.flush().unwrap();

    video_list
        .into_iter()
        .zip(is_selected)
        .filter(|(_, selected)| *selected)
        .map(|(video, _)| video)
        .collect()
}

fn print_video_list(
    buf: &mut RawTerminal<Stdout>,
    video_list: &[Task],
    is_selected: &[bool],
    line: usize,
) {
    write!(buf, "{}{}", clear::All, cursor::Goto(1, 1)).unwrap();
    for (i, (video, selected)) in video_list.iter().zip(is_selected.iter()).enumerate() {
        write!(buf, "{}", termion::cursor::Goto(1, i as u16 + 1)).unwrap();
        match (i == line, *selected) {
            (true, true) => writeln!(
                buf,
                "{} [✓] {}{}{} <<<{}",
                color::Fg(color::Rgb(
                    SELECTED_COLOR.0,
                    SELECTED_COLOR.1,
                    SELECTED_COLOR.2
                )),
                style::Underline,
                video.title,
                style::NoUnderline,
                color::Fg(color::Reset)
            )
            .unwrap(),
            (true, false) => writeln!(
                buf,
                "{} [ ] {}{}{} <<<{}",
                color::Fg(color::Rgb(
                    UNSELECTED_COLOR.0,
                    UNSELECTED_COLOR.1,
                    UNSELECTED_COLOR.2
                )),
                style::Underline,
                video.title,
                style::NoUnderline,
                color::Fg(color::Reset)
            )
            .unwrap(),
            (false, true) => writeln!(
                buf,
                "{} [✓] {}{}",
                color::Fg(color::Rgb(
                    SELECTED_COLOR.0,
                    SELECTED_COLOR.1,
                    SELECTED_COLOR.2
                )),
                video.title,
                color::Fg(color::Reset)
            )
            .unwrap(),
            (false, false) => writeln!(
                buf,
                "{} [ ] {}{}",
                color::Fg(color::Rgb(
                    UNSELECTED_COLOR.0,
                    UNSELECTED_COLOR.1,
                    UNSELECTED_COLOR.2
                )),
                video.title,
                color::Fg(color::Reset)
            )
            .unwrap(),
        }
    }

    let hint_line = video_list.len() as u16 + 2;
    write!(
        buf,
        "{} ↑/↓: 上下移动; d: 选择 / 取消选择; a: 全选 / 全不选; y: 确认; n: 取消下载{}",
        cursor::Goto(1, hint_line),
        cursor::Goto(1, hint_line + 1)
    )
    .unwrap();
}

fn termion_init(buf: &mut RawTerminal<Stdout>) {
    write!(
        buf,
        "{}{}{}",
        termion::clear::All,
        termion::cursor::Goto(1, 1),
        termion::cursor::Hide
    )
    .unwrap();
    buf.flush().unwrap();
}

pub fn wait() {
    println!("点击任意键继续...");
    let stdin = stdin();
    let _ = stdin.keys().next();
}

#[cfg(test)]
mod cli_test {
    use super::*;

    #[test]
    #[should_panic]
    fn cli_test1() {
        let cli = Cli::try_parse_from(["bili-dl", "invalid-url"].iter()).unwrap();
        println!("{:?}", cli);
    }

    #[test]
    fn cli_test2() {
        let cli = Cli::try_parse_from([
            "bili-dl",
            "https://www.bilibili.com/bangumi/media/md21231728",
        ]);
        assert_eq!(cli.unwrap().dl_dir, dirs::download_dir().unwrap());
    }
}
