use clap::Parser;
use reqwest::Url;

#[derive(Parser, Debug)]
#[command(name = "bili-dl")]
#[command(version = "0.1.0")]
pub struct Cli {
    /// The URL of the video
    #[arg(required = true, value_parser = validate_url)]
    pub url: Url,

    /// The path to the cookies file
    #[arg(long, short)]
    pub cookies: Option<String>,

    /// The directory to save the downloaded video
    #[arg(long, short)]
    pub dl_dir: Option<String>,

    /// The path to the ffmpeg binary
    #[arg(long, short)]
    pub ffmpeg: Option<String>,
}

fn validate_url(url: &str) -> Result<Url, String> {
    let u = Url::parse(url).expect("failed to parse url");
    Ok(u)
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

    // #[test]
    // fn cli_test2() {
    //     let cli = Cli::try_parse_from(["bili-dl", "h"]);
    //     println!("{:?}", cli);
    // }
}