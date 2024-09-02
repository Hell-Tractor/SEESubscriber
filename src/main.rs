use client::Lecture;
use log::{error, info, warn};
use utils::config;

mod utils;
mod data;
mod constants;
mod client;
mod login;

#[derive(thiserror::Error, Debug)]
enum Error {
    #[error(transparent)]
    ConfigError(#[from] config::ConfigError),
    #[error(transparent)]
    ClientError(#[from] client::Error),
}

type Result<T> = std::result::Result<T, Error>;

#[tokio::main]
async fn main() {
    pretty_env_logger::init_timed();
    info!("Starting...");
    if let Err(e) = work().await {
        error!("Error: {}", e);
    }
    info!("Done...");
}

async fn work() -> Result<()> {
    let client = client::Client::new();
    let pages = config().get_array("pages")?;
    let mut data = data::Data::load_or_default();
    let mut new_notices = Vec::new();
    for page in pages {
        let page = page.into_string();
        if let Err(e) = page {
            warn!("Failed to convert page to string: {}, skipped", e);
            continue;
        }
        let page = page.unwrap();

        let latest_notice = client.get_latest_notice_full_path(&page).await?;
        if data.get(&page).map_or(true, |old| *old != latest_notice.url) {
            info!("New notice found for page {}: {}, title: {}", page, latest_notice.url, latest_notice.title);
            data.set(&page, latest_notice.url.clone());
            new_notices.push(latest_notice);
        }
    }
    client.send_notice(&new_notices).await?;
    info!("{} new notice(s) found.", new_notices.len());

    info!("Getting new lectures...");
    let session_id = data.get("sessionid");
    let old_lectures: Vec<Lecture> = match data.get("lectures") {
        Some(lectures) => serde_json::from_str(lectures).unwrap_or(Vec::new()),
        None => Vec::new()
    };
    let (new_lectures, all_lectures, new_session_id) = client.get_new_lectures(old_lectures, &session_id).await?;
    client.send_lecture(&new_lectures).await?;
    data.set("lectures", serde_json::to_string(&all_lectures).unwrap());
    data.set("sessionid", new_session_id);
    info!("{} new lecture(s) found.", new_lectures.len());

    Ok(())
}