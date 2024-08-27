use log::{debug, info};
use reqwest::Client;

use super::Notice;

pub trait NoticeAdapter {
    async fn send_notice(client: &Client, notice: &[Notice]) -> super::Result<()>;
}

pub struct SCTAdapter;

impl NoticeAdapter for SCTAdapter {
    #[allow(unused_variables)]
    async fn send_notice(client: &Client, notice: &[Notice]) -> super::Result<()> {
        info!("Sending SCT message...");
        if notice.len() == 0 {
            info!("No new notice found, skipping...");
            return Ok(());
        }
        let sct_key = super::config().get_string("sct.key");
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