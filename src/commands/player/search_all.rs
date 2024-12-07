use songbird::input::Input;

use crate::{input::spotdl::SpotifyDl, Context, Error, HttpKey};

use super::join::handle_join;

#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn search_all(
    ctx: Context<'_>,
    #[description = "Url to the song"] url: String,
) -> Result<(), Error> {
    ctx.defer().await?;

    handle_search_all_song(ctx, url, 0, 2).await?;

    Ok(())
}

async fn handle_search_all_song(
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
    let ser_ctx = ctx.serenity_context();
    let guild_id = ctx.guild_id().expect("have guild_id");

    let http_client = {
        let data = ser_ctx.data.read().await;
        data.get::<HttpKey>()
            .cloned()
            .expect("Guaranteed to exist in the typemap.")
    };

    let manager = songbird::get(ser_ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    match manager.get(guild_id) {
        Some(handler_lock) => {
            let mut handler = handler_lock.lock().await;

            let src = SpotifyDl::new(http_client, url);

            let mut output: Input = src.clone().into();
            let raw_metadata = output.aux_metadata().await?;

            if let Some(metadata) = raw_metadata.source_url {
                println!("metadata {:?}", metadata);
            } else {
                println!("metadata is none");
            }

            handler.enqueue_input(src.into()).await;

            let q_len = handler.queue().len();
            println!("current queue length {}", q_len);

            ctx.reply("Playing song").await?;
        }
        _ => {
            ctx.reply("Not in a voice channel to play in, joining...")
                .await?;
            if let Ok(_) = handle_join(ctx).await {
                let future = Box::pin(handle_search_all_song(
                    ctx,
                    url,
                    trial_time + 1,
                    max_trial_time,
                ));
                future.await?;
            }
        }
    }
    Ok(())
}
