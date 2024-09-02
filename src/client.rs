use reqwest::header::{REFERER, USER_AGENT};
use tokio::try_join;

use crate::{constants, data, utils::config};

mod notice_adapter;

pub use notice_adapter::*;

pub struct Client(reqwest::Client);

pub struct Notice {
    pub title: String,
    pub url: String
}

#[derive(serde::Deserialize)]
pub struct Lecture {
    #[serde(rename = "cathedra")]
    pub title: String,
    #[serde(rename = "classLevelName")]
    pub level: String,
    #[serde(rename = "lectureTime")]
    pub time: String,
    #[serde(rename = "lectureId")]
    pub id: String,
    #[serde(rename = "nameSpeaker")]
    pub speaker: String,
}

#[derive(serde::Deserialize)]
struct LectureListVo {
    code: i32,
    msg: String,
    data: Vec<Lecture>,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    #[error(transparent)]
    ConfigError(#[from] config::ConfigError),
    #[error(transparent)]
    LocalNotificationError(#[from] notify_rust::error::Error),
    #[error("No element found with selector: {0}")]
    ElementNotFound(String),
    #[error(transparent)]
    SerdeJsonError(#[from] serde_json::Error),
    #[error("Unknown error: {0}")]
    UnknownError(String),
}

type Result<T> = std::result::Result<T, Error>;

impl Client {
    pub fn new() -> Self {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(USER_AGENT, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/127.0.0.0 Safari/537.36".parse().unwrap());
        headers.insert(REFERER, "https://see.tongji.edu.cn/index.htm".parse().unwrap());

        Client(reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .unwrap())
    }

    pub async fn get_latest_notice_full_path(&self, sub_path: &str) -> Result<Notice> {
        let base_url = config().get_string("url")?;
        let url = format!("{}/{}", base_url, sub_path);
        let response = self.0.get(&url).send().await?.text().await?;
        let document = scraper::Html::parse_document(&response);
        document.select(&scraper::Selector::parse(constants::NOTICE_SELECTOR).unwrap()).next()
            .ok_or_else(|| Error::ElementNotFound(constants::NOTICE_SELECTOR.to_string()))
            .and_then(|element| {
                let href = element.value().attr("href");
                let title = element.value().attr("title");
                if href.is_none() || title.is_none() {
                    return Err(Error::ElementNotFound(constants::NOTICE_SELECTOR.to_string()));
                }
                Ok(Notice {
                    title: title.unwrap().to_string(),
                    url: format!("{}/{}", base_url, href.unwrap())
                })
            })
    }

    pub async fn get_new_lectures(&self, last_lecutre_id: Option<&str>) -> Result<Vec<Lecture>> {
        let url = config().get_string("lecture_url")?;
        let session_id = config().get_string("session_id")?;

        let response = self.0.get(&url)
            .header("Cookie", format!("sessionid={}", session_id))
            .header("Referer", "https://1.tongji.edu.cn/workbench")
            .send().await?
            .text().await?;
        let lecture_list: LectureListVo = serde_json::from_str(&response)?;
        if lecture_list.code != 200 {
            return Err(Error::UnknownError(lecture_list.msg));
        }

        Ok(lecture_list.data.into_iter()
            .skip_while(|lecture| last_lecutre_id.map_or(false, |id| lecture.id == id))
            .filter(|lecture| last_lecutre_id.map_or(false, |id| lecture.id != id))
            .collect::<Vec<_>>())
    }

    pub async fn send_notice(&self, notice: &[Notice]) -> Result<()> {
        try_join!(
            LocalAdapter::send_notice(&self.0, notice),
            SCTAdapter::send_notice(&self.0, notice)
        ).map(|_| ())
    }

    pub async fn send_lecture(&self, lectures: &[Lecture]) -> Result<()> {
        try_join!(
            LocalAdapter::send_lecture(&self.0, lectures),
            SCTAdapter::send_lecture(&self.0, lectures)
        ).map(|_| ())
    }
}