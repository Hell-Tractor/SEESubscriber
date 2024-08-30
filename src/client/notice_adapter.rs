use reqwest::Client;

use super::Notice;

mod adapters;
pub use adapters::*;

pub trait NoticeAdapter {
    async fn send_notice(client: &Client, notice: &[Notice]) -> super::Result<()>;
}