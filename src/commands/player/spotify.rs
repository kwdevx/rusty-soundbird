use songbird::input::Compose;
use crate::{
    input::sources::spotdl::{SpotifyCredential, SpotifyDl},
    Context, Error, HttpKey,
};

use super::join::handle_join;

#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn spotify(
    ctx: Context<'_>,
    #[description = "Url to the song"] url: String,
) -> Result<(), Error> {
    ctx.defer().await?;

    handle_play_spotify(ctx, url, 0, 2).await?;

    Ok(())
}

async fn handle_play_spotify(
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

            let src = SpotifyDl::new(
                http_client,
                url,
                Some(SpotifyCredential {
                    client_id: ctx.data().app_config.spotify_client_id.clone(),
                    client_secret: ctx.data().app_config.spotify_client_secret.clone(),
                }),
            );

            handler.enqueue_input(src.into()).await;

            let q_len = handler.queue().len();
            println!("current queue length {}", q_len);

            // handle metadata for spotify adaptor
            ctx.reply("Playing song").await?;
        }
        _ => {
            ctx.reply("Not in a voice channel to play in, joining...")
                .await?;
            if let Ok(_) = handle_join(ctx).await {
                let future = Box::pin(handle_play_spotify(
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
