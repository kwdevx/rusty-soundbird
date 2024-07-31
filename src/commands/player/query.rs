use crate::{Context, Error, HttpKey};
use poise::serenity_prelude::Message;
use serde::{Deserialize, Serialize};
use songbird::input::YoutubeDl;

#[poise::command(
    context_menu_command = "Query song",
    prefix_command,
    track_edits,
    slash_command
)]
pub async fn query(
    ctx: Context<'_>,
    #[description = "Url to the song"] url: Message,
) -> Result<(), Error> {
    ctx.defer().await?;

    // handle_query_song(ctx, url.content, 0, 2).await?;

    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
struct QueryResult {
    name: String,
    value: String,
}

async fn handle_query_song(
    ctx: Context<'_>,
    url: String,
    trial_time: i8,
    max_trial_time: i8,
) -> Result<(), Error> {
    if trial_time >= max_trial_time {
        ctx.reply("Tried to join the channle multiple times but fail")
            .await?;
        return Ok(());
    }

    let do_search = !url.starts_with("http");
    let ser_ctx = ctx.serenity_context();
    // let guild_id = ctx.guild_id().expect("have guild_id");

    let http_client = {
        let data = ser_ctx.data.read().await;
        data.get::<HttpKey>()
            .cloned()
            .expect("Guaranteed to exist in the typemap.")
    };

    if do_search {
        println!("searching...");
        let search_res = YoutubeDl::new_search(http_client.clone(), url.clone())
            .search(Some(5))
            .await?;

        println!("search res length {}", search_res.len());

        let mut command_res: Vec<QueryResult> = Vec::new();

        for res in search_res {
            match (res.title, res.source_url) {
                (Some(title), Some(url)) => command_res.push(QueryResult {
                    name: title,
                    value: url,
                }),
                _ => {}
            }
        }

        if let Ok(val) = serde_json::to_string(&command_res) {
            ctx.reply(val).await?;
        }
    }

    Ok(())
}
