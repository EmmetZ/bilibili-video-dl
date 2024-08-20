use clap::Parser;
use reqwest::Url;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "bili-dl")]
#[command(version = "1.3.0")]
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
