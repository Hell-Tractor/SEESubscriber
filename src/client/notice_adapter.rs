use reqwest::Client;

use super::{Lecture, Notice};

mod adapters;
pub use adapters::*;

pub trait NoticeAdapter {
    async fn send_notice(client: &Client, notice: &[Notice]) -> super::Result<()>;
    async fn send_lecture(client: &Client, lecture: &[Lecture]) -> super::Result<()>;
}