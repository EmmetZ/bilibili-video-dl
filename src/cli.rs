use clap::Parser;
use reqwest::Url;
use std::path::{self, PathBuf};

#[derive(Parser, Debug)]
#[command(name = "bili-dl")]
#[command(version = "1.4.0")]
pub struct Cli {
    /// 视频/番剧链接
    #[arg(value_parser = validate_url, value_name = "url")]
    pub url: Url,

    /// 下载目录路径
    #[arg(value_parser = set_dir, default_value = "./", value_name = "path")]
    pub dl_dir: PathBuf,

    /// cookies.txt 的路径
    #[arg(long, short, value_name = "path")]
    pub cookies: Option<String>,
}

fn validate_url(url: &str) -> Result<Url, String> {
    let u = Url::parse(url).expect("链接格式错误");
    Ok(u)
}

fn set_dir(dir: &str) -> Result<PathBuf, String> {
    let d = path::absolute(dir).unwrap().canonicalize().unwrap();

    if !d.exists() || !d.is_dir() {
        return Err("文件夹不存在".into());
    }
    // println!("{:?}", d);
    Ok(d)
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

    #[test]
    fn cli_test3() {
        let cli = Cli::try_parse_from([
            "bili-dl",
            "https://www.bilibili.com/bangumi/media/md21231728",
        ]);
        println!("{:?}", cli.as_ref().unwrap().dl_dir);
        assert_eq!(cli.unwrap().dl_dir, std::env::current_dir().unwrap());
    }

    #[test]
    fn cli_test4() {
        let cli = Cli::try_parse_from([
            "bili-dl",
            "https://www.bilibili.com/bangumi/media/md21231728",
            "../",
        ]);
        println!("{:?}", cli.as_ref().unwrap().dl_dir);
        assert_eq!(
            cli.unwrap().dl_dir,
            std::env::current_dir().unwrap().parent().unwrap()
        );
    }
}
