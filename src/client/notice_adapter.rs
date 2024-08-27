use reqwest::Client;

pub trait NoticeAdapter {
    async fn send_notice(client: &Client, message: &str) -> super::Result<()>;
}