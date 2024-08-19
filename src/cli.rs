use crate::http::download::Task;
use clap::Parser;
use crossterm::{
    cursor,
    event::{read, Event, KeyCode},
    execute,
    style::{Attribute, Color, SetAttribute, SetForegroundColor},
    terminal::{self, Clear, ClearType},
};
use reqwest::Url;
use std::{
    io::{stdout, Stdout, Write},
    path::PathBuf,
};

const SELECTED_COLOR: Color = Color::Rgb {
    r: 255,
    g: 167,
    b: 38,
};

const UNSELECTED_COLOR: Color = Color::Rgb {
    r: 120,
    g: 144,
    b: 156,
};

#[derive(Parser, Debug)]
#[command(name = "bili-dl")]
#[command(version = "1.2")]
pub struct Cli {
    /// 视频链接
    #[arg(value_parser = validate_url)]
    pub url: Url,

    /// cookies.txt 的路径
    #[arg(long, short)]
    pub cookies: Option<String>,

    /// 下载目录，默认为当前目录
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
    terminal::enable_raw_mode().expect("Failed to enable raw mode");
    let mut buf = stdout();

    let mut flag = true;
    let mut is_selected = vec![flag; video_list.len()];
    let mut line: usize = 0;

    terminal_init(&mut buf);

    print_video_list(&mut buf, &video_list, &is_selected, line);
    buf.flush().unwrap();

    loop {
        if let Event::Key(k) = read().expect("Failed to read input") {
            let key_code = k.code;
            match key_code {
                KeyCode::Up => {
                    line = line.saturating_sub(1);
                }
                KeyCode::Down => {
                    if line < video_list.len() - 1 {
                        line += 1
                    }
                }
                KeyCode::Char('d') => {
                    is_selected[line] = !is_selected[line];
                }
                KeyCode::Enter => {
                    break;
                }
                KeyCode::Char('n') => {
                    println!("取消下载");
                    is_selected = vec![false; video_list.len()];
                    break;
                }
                KeyCode::Char('a') => {
                    flag = !flag;
                    is_selected = vec![flag; video_list.len()];
                }
                _ => {}
            }
            print_video_list(&mut buf, &video_list, &is_selected, line);
            buf.flush().unwrap();
        }
    }
    terminal::disable_raw_mode().expect("Failed to disable raw mode");
    execute!(buf, cursor::Show).unwrap();
    video_list
        .into_iter()
        .zip(is_selected)
        .filter(|(_, selected)| *selected)
        .map(|(video, _)| video)
        .collect()
}

fn print_video_list(buf: &mut Stdout, video_list: &[Task], is_selected: &[bool], line: usize) {
    execute!(buf, Clear(ClearType::All), cursor::MoveTo(0, 0)).unwrap();
    for (i, (video, selected)) in video_list.iter().zip(is_selected.iter()).enumerate() {
        write!(buf, "{}", cursor::MoveTo(0, i as _)).unwrap();
        match (i == line, *selected) {
            (true, true) => writeln!(
                buf,
                "{} [✓] {}{}{} <<<{}",
                SetForegroundColor(SELECTED_COLOR),
                SetAttribute(Attribute::Underlined),
                video.title,
                SetAttribute(Attribute::NoUnderline),
                SetForegroundColor(Color::Reset),
            )
            .unwrap(),
            (true, false) => writeln!(
                buf,
                "{} [ ] {}{}{} <<<{}",
                SetForegroundColor(UNSELECTED_COLOR),
                SetAttribute(Attribute::Underlined),
                video.title,
                SetAttribute(Attribute::NoUnderline),
                SetForegroundColor(Color::Reset),
            )
            .unwrap(),
            (false, true) => writeln!(
                buf,
                "{} [✓] {}{}",
                SetForegroundColor(SELECTED_COLOR),
                video.title,
                SetForegroundColor(Color::Reset),
            )
            .unwrap(),
            (false, false) => writeln!(
                buf,
                "{} [ ] {}{}",
                SetForegroundColor(UNSELECTED_COLOR),
                video.title,
                SetForegroundColor(Color::Reset),
            )
            .unwrap(),
        }
    }

    let hint_line = video_list.len() as u16 + 1;
    writeln!(
        buf,
        "{} ↑/↓: 上下移动; d: 选择 / 取消选择; a: 全选 / 全不选; Enter: 确认; n: 取消下载{}",
        cursor::MoveTo(0, hint_line),
        cursor::MoveTo(0, hint_line + 1)
    )
    .unwrap();
}

fn terminal_init(buf: &mut Stdout) {
    execute!(
        buf,
        terminal::Clear(terminal::ClearType::All),
        cursor::MoveTo(0, 0),
        cursor::Hide
    )
    .unwrap();
}

pub fn wait() {
    println!("点击任意键继续...");
    let _ = read().unwrap();
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
