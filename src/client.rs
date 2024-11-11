use std::sync::Arc;

use log::{debug, info, warn};
use reqwest::{cookie::Jar, header::{REFERER, USER_AGENT}, redirect, Url};
use tokio::try_join;

use crate::{constants, login, utils::config};

mod notice_adapter;

pub use notice_adapter::*;

pub struct Client {
    client: reqwest::Client,
    cookie_jar: Arc<Jar>,
}

pub struct Notice {
    pub title: String,
    pub url: String
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
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

impl PartialEq for Lecture {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
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
    #[error("Failed to deserialize JSON: {0}. Text: {1}")]
    SerdeJsonError(serde_json::Error, String),
    #[error("Unknown error: {0}")]
    UnknownError(String),
    #[error(transparent)]
    LoginError(#[from] login::Error)
}

type Result<T> = std::result::Result<T, Error>;

impl Client {
    pub fn new() -> Self {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(USER_AGENT, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/127.0.0.0 Safari/537.36".parse().unwrap());
        headers.insert(REFERER, "https://see.tongji.edu.cn/index.htm".parse().unwrap());

        let cookie_jar = Arc::new(Jar::default());

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .cookie_provider(cookie_jar.clone())
            .redirect(redirect::Policy::limited(32))
            .build()
            .unwrap();

        Client { client, cookie_jar }
    }

    pub async fn get_latest_notice_full_path(&self, sub_path: &str) -> Result<Notice> {
        let base_url = config().get_string("school_notice.url")?;
        let url = format!("{}/{}", base_url, sub_path);
        let response = self.client.get(&url).send().await?.text().await?;
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

    fn get_lecture_date(lecture: &Lecture) -> Option<chrono::NaiveDate> {
        let date= lecture.time.split(" ").next()
            .and_then(|date| chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d").ok());
        if date == None {
            warn!("Failed to deserialize date `{}` in lecture(id = {})", lecture.time, lecture.id);
        }
        date
    }

    async fn remove_outdated_lectures(old_lectures: Vec<Lecture>) -> Vec<Lecture> {
        let today = chrono::Local::now().date_naive();
        old_lectures.into_iter().filter_map(|lecture| Client::get_lecture_date(&lecture).and_then(|date| Some((lecture, date))))
            .filter(|(_, date)| *date >= today)
            .map(|(lecture, _)| lecture)
            .collect()
    }

    /// return: (new, all_valid, sessionid)
    pub async fn get_new_lectures(&self, old_lectures: Vec<Lecture>, session_id: &Option<&str>) -> Result<(Vec<Lecture>, Vec<Lecture>, String)> {
        let old_lectures = Client::remove_outdated_lectures(old_lectures);
        let url = config().get_string("lecture.url")?;
        let timestamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis();
        let username = config().get_string("login.username")?;
        let password = config().get_string("login.password")?;

        let mut new_session_id = match session_id {
            Some(id) => {
                info!("session id found in cache, login with it.");
                self.cookie_jar.add_cookie_str(format!("sessionid={}", id).as_str(), &Url::parse("https://1.tongji.edu.cn/").unwrap());
                id.to_string()
            },
            None => {
                info!("session id not found in cache, login with username and password");
                login::login(&self.client, &username, &password).await?
            }
        };

        let mut response = self.client.get(&url)
            .header("Referer", "https://1.tongji.edu.cn/workbench")
            .query(&[("_t", timestamp)])
            .send().await?;
        if let Err(e) = response.error_for_status_ref() {
            info!("Failed to get lecture list, re-login and retry. Msg: {}", e);
            new_session_id = login::login(&self.client, &username, &password).await?;
            response = self.client.get(&url)
                .header("Referer", "https://1.tongji.edu.cn/workbench")
                .query(&[("_t", timestamp)])
                .send().await?;
        }
        let response = response.text().await?;
        debug!("Lecture response: {}", response);
        let lecture_list: LectureListVo = serde_json::from_str(&response).map_err(|e| Error::SerdeJsonError(e, response))?;
        if lecture_list.code != 200 {
            return Err(Error::UnknownError(lecture_list.msg));
        }
        let current_lectures = lecture_list.data;
        let mut old_lectures = old_lectures.await;
        let mut new_lectures = Vec::new();
        for current_lecture in current_lectures {
            if !old_lectures.contains(&current_lecture) {
                old_lectures.push(current_lecture.clone());
                new_lectures.push(current_lecture);
            }
        }
        Ok((new_lectures, old_lectures, new_session_id))
    }

    pub async fn send_notice(&self, notice: &[Notice]) -> Result<()> {
        try_join!(
            LocalAdapter::send_notice(&self.client, notice),
            SCTAdapter::send_notice(&self.client, notice),
            SC3Adapter::send_notice(&self.client, notice)
        ).map(|_| ())
    }

    pub async fn send_lecture(&self, lectures: &[Lecture]) -> Result<()> {
        try_join!(
            LocalAdapter::send_lecture(&self.client, lectures),
            SCTAdapter::send_lecture(&self.client, lectures),
            SC3Adapter::send_lecture(&self.client, lectures)
        ).map(|_| ())
    }

    pub async fn report_error(&self, message: &str, err: &crate::Error) -> Result<()> {
        try_join!(
            LocalAdapter::report_error(&self.client, message, err),
            SCTAdapter::report_error(&self.client, message, err),
            SC3Adapter::report_error(&self.client, message, err)
        ).map(|_| ())
    }
}