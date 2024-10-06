use log::info;
use reqwest::Client;

use super::NoticeAdapter;
use super::super::Notice;
use super::super::Result;
use super::is_configured;

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