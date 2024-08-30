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

impl NoticeAdapter for LocalAdapter {
    async fn send_notice(_client: &Client, notice: &[Notice]) -> Result<()> {
        if !is_configured("local") {
            info!("LocalAdapter not configured, skipping...");
            return Ok(());
        }
        Notification::new()
            .summary(format!("学院已发布{}条新的通知/公告", notice.len()).as_str())
            .body(notice.iter().map(|n| format!("- {}", n.title)).collect::<Vec<String>>().join("\n").as_str())
            .timeout(Timeout::Milliseconds(3000))
            .show().map_err(|e| e.into())
    }
}

pub struct SCTAdapter;

impl NoticeAdapter for SCTAdapter {
    async fn send_notice(client: &Client, notice: &[Notice]) -> Result<()> {
        if !is_configured("SCT") {
            info!("SCTAdapter not configured, skipping...");
            return Ok(());
        }
        info!("Sending SCT message...");
        if notice.len() == 0 {
            info!("No new notice found, skipping...");
            return Ok(());
        }
        let sct_key = crate::config().get_string("sct.key");
        if let Err(config::ConfigError::NotFound(_)) = sct_key {
            info!("SCT key not found, skipping...");
            return Ok(());
        }
        let sct_key = sct_key?;
        let url = format!("https://sctapi.ftqq.com/{}.send", sct_key);
        let desp = format!("# 通知/公告列表\n\n{}", notice.iter().map(|n| format!("- [{}]({})", n.title, n.url)).collect::<Vec<String>>().join("\n"));
        let short = format!(r#""{}"等{}条通知/公告"#, notice[0].title, notice.len());
        let response = client.post(&url)
            .form(&[("title", "学院已发布新的通知/公告"), ("desp", desp.as_str()), ("short", short.as_str())])
            .send().await?;
        debug!("SCT url: {url}, SCT message: title: 学院已发布新的通知/公告, desp: {desp}, short: {short}");
        info!("SCT message sent.");
        response.error_for_status().map_err(|e| e.into()).map(|_| ())
    }
}