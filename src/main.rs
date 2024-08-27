use log::{error, info, warn};
use utils::config;

mod utils;
mod data;
mod constants;
mod client;

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
    let mut count = 0;
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
            count += 1;
            data.set(&page, latest_notice.url);
        }
    }
    info!("{} new notices found.", count);

    Ok(())
}