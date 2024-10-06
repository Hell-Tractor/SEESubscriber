use reqwest::Client;

use super::{Lecture, Notice};

mod local_adapter;
pub use local_adapter::LocalAdapter;
mod sct_adapter;
pub use sct_adapter::SCTAdapter;
mod sc3_adapter;
pub use sc3_adapter::SC3Adapter;

pub trait NoticeAdapter {
    async fn send_notice(client: &Client, notice: &[Notice]) -> super::Result<()>;
    async fn send_lecture(client: &Client, lecture: &[Lecture]) -> super::Result<()>;
}

fn is_configured(key: &str) -> bool {
    crate::config().get_array("notice").map(|array| array.into_iter().filter_map(|v| v.into_string().ok()).any(|v| v.to_lowercase() == key.to_lowercase())).unwrap_or(false)
}