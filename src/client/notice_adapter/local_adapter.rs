use log::info;
use notify_rust::Notification;
use notify_rust::Timeout;
use reqwest::Client;

use super::NoticeAdapter;
use super::super::Notice;
use super::super::Result;
use super::is_configured;

pub struct LocalAdapter;

impl LocalAdapter {
    async fn send_msg(title: &str, body: &str) -> Result<()> {
        info!("Sending local message...");
        if !is_configured("local") {
            info!("LocalAdapter not configured, skipping...");
            return Ok(());
        }
        Notification::new()
            .summary(title)
            .body(body)
            .timeout(Timeout::Milliseconds(3000))
            .show()
            .map(|_| ())
            .map_err(|e| e.into())
    }
}

impl NoticeAdapter for LocalAdapter {
    async fn send_notice(_client: &Client, notice: &[Notice]) -> Result<()> {
        let title = format!("学院已发布{}条新的通知/公告", notice.len());
        let body = notice.iter().map(|n| format!("- {}", n.title)).collect::<Vec<String>>().join("\n");
        LocalAdapter::send_msg(&title, &body).await
    }

    async fn send_lecture(_client: &Client, lecture: &[crate::client::Lecture]) -> Result<()> {
        let title = format!("找到{}条新的同济大讲堂", lecture.len());
        let body = lecture.iter().map(|l| format!("- {}", l.title)).collect::<Vec<String>>().join("\n");
        LocalAdapter::send_msg(&title, &body).await
    }
}