use log::info;
use reqwest::Client;

use super::NoticeAdapter;
use super::super::Notice;
use super::super::Result;
use super::is_configured;

pub struct SC3Adapter;

impl SC3Adapter {
    async fn send_msg(client: &Client, title: &str, desp: &str, short: &str, tags: &str) -> Result<()> {
        info!("Sending SC3 message...");
        if !is_configured("SC3") {
            info!("SC3Adapter not configured, skipping...");
            return Ok(());
        }
        let sc3_key = crate::config().get_string("sc3.key");
        if let Err(config::ConfigError::NotFound(_)) = sc3_key {
            info!("SC3 key not found, skipping...");
            return Ok(());
        }
        let sct_key = sc3_key?;
        let url = format!("https://{}.push.ft07.com/send", sct_key);
        let response = client.post(&url)
            .form(&[("title", title), ("desp", desp), ("short", short), ("tags", tags)])
            .send().await?;
        info!("SC3 message sent.");
        response.error_for_status().map_err(|e| e.into()).map(|_| ())
    }
}

impl NoticeAdapter for SC3Adapter {
    async fn send_notice(client: &Client, notice: &[Notice]) -> Result<()> {
        if notice.len() == 0 {
            info!("No new notice found, skipping...");
            return Ok(());
        }
        let desp = format!("# 通知/公告列表\n\n{}", notice.iter().map(|n| format!("- [{}]({})", n.title, n.url)).collect::<Vec<String>>().join("\n"));
        let short = format!(r#""{}"等{}条通知/公告"#, notice[0].title, notice.len());
        SC3Adapter::send_msg(client, "学院已发布新的通知/公告", &desp, &short, "同济大学|通知").await
    }

    async fn send_lecture(client: &Client, lecture: &[crate::client::Lecture]) -> crate::client::Result<()> {
        if lecture.len() == 0 {
            info!("No new lecture found, skipping...");
            return Ok(());
        }
        let desp = format!("|主题|级别|主讲人|时间|\n|:-:|:-:|:-:|:-:|\n{}", lecture.iter().map(|l| format!("|{}|{}|{}|{}|", l.title, l.level, l.speaker, l.time)).collect::<Vec<String>>().join("\n"));
        let short = format!(r#""{}"等{}条同济大讲堂"#, lecture[0].title, lecture.len());
        SC3Adapter::send_msg(client, "找到新的同济大讲堂", &desp, &short, "同济大学|同济大讲堂").await
    }
}