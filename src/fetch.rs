use reqwest::{
    header::{HeaderMap, COOKIE, REFERER, USER_AGENT},
    Client,
};
use std::{error::Error, fs, time::Duration};

pub fn init_default_header(url: &str, cookies: Option<&String>) -> HeaderMap {
    let mut header = HeaderMap::new();
    let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/127.0.0.0 Safari/537.36 Edg/127.0.0.0";
    header.insert(USER_AGENT, ua.parse().unwrap());
    header.insert(REFERER, url.parse().unwrap());
    if let Some(c) = cookies {
        header.insert(COOKIE, c.parse().unwrap());
    }
    header
}

pub fn encode_cookies(c_path: String) -> Result<Option<String>, Box<dyn Error>> {
    let s = fs::read_to_string(c_path).expect("failed to read cookies file");
    let lines: Vec<&str> = s.lines().collect();
    let cookies = lines.join("; ");
    // println!("{}", cookies);
    if cookies.is_empty() {
        println!("cookies is empty");
        Ok(None)
    } else {
        println!("successfully read cookies");
        Ok(Some(cookies))
    }
}

pub async fn fetch_url(url: &str, cookies: Option<&String>) -> Result<String, Box<dyn Error>> {
    let client = reqwest::Client::new();
    let _ = validate_login(&client, cookies).await?;
    let resp = client
        .get(url)
        .headers(init_default_header(url, cookies))
        .timeout(Duration::from_secs(3))
        .send()
        .await?;

    let body = resp.text().await?;
    Ok(body)
}

async fn validate_login(client: &Client, cookies: Option<&String>) -> Result<bool, Box<dyn Error>> {
    let url = "https://api.bilibili.com/x/web-interface/nav";
    let resp = client
        .get(url)
        .headers(init_default_header(url, cookies))
        .timeout(Duration::from_secs(3))
        .send()
        .await?;

    // Check if the response contains a login indicator
    let body = resp.text().await?;
    let is_logged_in = body.contains("\"isLogin\":true");

    if is_logged_in {
        println!("Successfully logged in\n");
    } else {
        println!("Not logged in\n");
    }

    Ok(is_logged_in)
}

#[cfg(test)]
mod fetch_test {
    use super::*;
    use reqwest::Url;
    use std::path::Path;

    #[tokio::test]
    async fn fetch_test() {
        let url = "https://www.bilibili.com/video/BV1Cs4y117Mu/?spm_id_from=333.337.search-card.all.click&vd_source=7b61f7ca2c7edcd57c0ffd1c17ee4e4c";
        let body = fetch_url(url, None).await.expect("1");
        println!("{}", body);
    }

    #[test]
    fn url_test() {
        let url = "https://upos-sz-estgoss.bilivideo.com/upgcxcode/92/57/1094305792/1094305792_nb3-1-30280.m4s?e=ig8euxZM2rNcNbdlhoNvNC8BqJIzNbfqXBvEqxTEto8BTrNvN0GvT90W5JZMkX_YN0MvXg8gNEV4NC8xNEV4N03eN0B5tZlqNxTEto8BTrNvNeZVuJ10Kj_g2UB02J0mN0B5tZlqNCNEto8BTrNvNC7MTX502C8f2jmMQJ6mqF2fka1mqx6gqj0eN0B599M=&uipk=5&nbs=1&deadline=1723464725&gen=playurlv2&os=upos&oi=1964813008&trid=caad2d19bba34b5c9d34eaa24e9c5841u&mid=398839362&platform=pc&og=cos&upsig=116974f3336c0b09b59913ea08eb383b&uparams=e,uipk,nbs,deadline,gen,os,oi,trid,mid,platform,og&bvc=vod&nettype=0&orderid=1,3&buvid=9572C95D-A944-1C55-26EE-4962F985BD6E28033infoc&build=0&f=u_0_0&agrr=1&bw=11003&logo=40000000";
        let u = Url::parse(url).unwrap();
        let path = u.path();
        let p = Path::new(path);
        println!("{:?}", p.file_stem());

        let extension = p.extension().unwrap().to_str().unwrap();
        println!("{}", extension);
    }

    #[test]
    fn cookies_test() {
        let c_path = "cookies.txt".to_string();
        let a = encode_cookies(c_path).unwrap();
        println!("{a:?}");
    }

    #[tokio::test]
    async fn login_test() {
        let client = reqwest::Client::new();
        let res = validate_login(&client, None).await;
        assert_eq!(res.unwrap(), false);
    }
}
