use std::time::Duration;

use reqwest::{Client, IntoUrl, Url};
use serde::{Deserialize, de::DeserializeOwned};

use crate::{
    asc_loader::AscModule,
    tg_api::{SendMessage, Update, UpdateResponse},
};

pub async fn send_msg(
    client: &Client,
    tg_key: &str,
    chat_id: i64,
    msg: String,
) -> anyhow::Result<()> {
    tracing::info!("Send msg called");
    client
        .post(format!("https://api.telegram.org/bot{tg_key}/sendMessage"))
        .json(&SendMessage { chat_id, text: msg })
        .send()
        .await?;
    Ok(())
}

pub async fn serve(mut module: AscModule, tg_key: String) {
    tracing::info!("Started serving");
    let client = Client::new();
    let mut errors = 0;
    let mut update_offset = 0;
    loop {
        match get_json::<UpdateResponse>(
            &client,
            &format!("https://api.telegram.org/bot{tg_key}/getUpdates?timeout=60&offset={update_offset}"),
        )
        .await
        {
            Ok(updates) => {
                tracing::info!(?updates, "Got updates");
                errors = 0;
                for u in updates.result {
                    update_offset = update_offset.max(u.update_id + 1);
                    if let Err(e) = process_update(&mut module, u).await {
                        tracing::error!(?e, "When processing update");
                    }
                }
            }
            Err(e) => {
                errors += 1;
                tracing::error!(?e, "when getting updates");
                let sleep_time_sec = (1 << errors).min(60);
                tokio::time::sleep(Duration::from_secs(sleep_time_sec)).await
            }
        }
    }
}

async fn process_update(module: &mut AscModule, u: Update) -> anyhow::Result<()> {
    tracing::info!("Calling process updates");
    module.call_process_updates(serde_json::to_string(&u)?).await?;
    Ok(())
}

async fn get_json<'a, T: DeserializeOwned>(
    client: &Client,
    url: impl IntoUrl,
) -> anyhow::Result<T> {
    Ok(client.get(url).send().await?.json().await?)
}
