use std::{fs, path::PathBuf};

use clap::Parser;
use reqwest::Url;

#[derive(Parser, Debug)]
#[command(name = "bili-dl")]
#[command(version = "1.0.0")]
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
        match dirs::download_dir() {
            Some(d) => Ok(d),
            None => {
                let d = dirs::home_dir().unwrap().join("Downloads");
                fs::create_dir_all(&d).expect("下载文件夹新建失败");
                // println!("下载路径: {}", d.display());
                Ok(d)
            }
        }
    }
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
