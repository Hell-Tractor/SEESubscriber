use log::{debug, info};
use notify_rust::Notification;
use notify_rust::Timeout;
use reqwest::Client;

use super::NoticeAdapter;
use super::super::Notice;
use super::super::Result;

fn is_configured(key: &str) -> bool {
    crate::config().get_array("notice").map(|array| array.into_iter().filter_map(|v| v.into_string().ok()).any(|v| v.to_lowercase() == key.to_lowercase())).unwrap_or(false)
}

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

pub struct SCTAdapter;

impl SCTAdapter {
    async fn send_msg(client: &Client, title: &str, desp: &str, short: &str) -> Result<()> {
        info!("Sending SCT message...");
        if !is_configured("SCT") {
            info!("SCTAdapter not configured, skipping...");
            return Ok(());
        }
        let sct_key = crate::config().get_string("sct.key");
        if let Err(config::ConfigError::NotFound(_)) = sct_key {
            info!("SCT key not found, skipping...");
            return Ok(());
        }
        let sct_key = sct_key?;
        let url = format!("https://sctapi.ftqq.com/{}.send", sct_key);
        let response = client.post(&url)
            .form(&[("title", title), ("desp", desp), ("short", short)])
            .send().await?;
        debug!("SCT url: {url}, SCT message: title: {title}, desp: {desp}, short: {short}");
        info!("SCT message sent.");
        response.error_for_status().map_err(|e| e.into()).map(|_| ())
    }
}

impl NoticeAdapter for SCTAdapter {
    async fn send_notice(client: &Client, notice: &[Notice]) -> Result<()> {
        if notice.len() == 0 {
            info!("No new notice found, skipping...");
            return Ok(());
        }
        let desp = format!("# 通知/公告列表\n\n{}", notice.iter().map(|n| format!("- [{}]({})", n.title, n.url)).collect::<Vec<String>>().join("\n"));
        let short = format!(r#""{}"等{}条通知/公告"#, notice[0].title, notice.len());
        SCTAdapter::send_msg(client, "学院已发布新的通知/公告", &desp, &short).await
    }

    async fn send_lecture(client: &Client, lecture: &[crate::client::Lecture]) -> crate::client::Result<()> {
        if lecture.len() == 0 {
            info!("No new lecture found, skipping...");
            return Ok(());
        }
        let desp = format!("|主题|级别|主讲人|时间|\n|:-:|:-:|:-:|:-:|\n{}", lecture.iter().map(|l| format!("|{}|{}|{}|{}|", l.title, l.level, l.speaker, l.time)).collect::<Vec<String>>().join("\n"));
        let short = format!(r#""{}"等{}条同济大讲堂"#, lecture[0].title, lecture.len());
        SCTAdapter::send_msg(client, "找到新的同济大讲堂", &desp, &short).await
    }
}