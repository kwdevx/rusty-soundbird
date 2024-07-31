use crate::{Context, Error, HttpKey};
use songbird::input::YoutubeDl;

use super::join::handle_join;

#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn play(
    ctx: Context<'_>,
    #[description = "Url to the song"] url: String,
) -> Result<(), Error> {
    ctx.defer().await?;

    handle_play_song(ctx, url, 0, 2).await?;

    Ok(())
}

async fn handle_play_song(
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
    let guild_id = ctx.guild_id().expect("have guild_id");

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

        for res in search_res {
            match (res.title) {
                (Some(title)) => {
                    println!("title:{title}");
                }
                _ => {}
            }
        }
    }

    let manager = songbird::get(ser_ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    match manager.get(guild_id) {
        Some(handler_lock) => {
            let mut handler = handler_lock.lock().await;

            let src = if do_search {
                YoutubeDl::new_search(http_client, url)
            } else {
                YoutubeDl::new(http_client, url)
            };

            let _ = handler.play_input(src.clone().into());

            ctx.reply("Playing song").await?;
        }
        _ => {
            ctx.reply("Not in a voice channel to play in, joining...")
                .await?;
            if let Ok(_) = handle_join(ctx).await {
                let future = Box::pin(handle_play_song(ctx, url, trial_time + 1, max_trial_time));
                future.await?;
            }
        }
    }
    Ok(())
}
