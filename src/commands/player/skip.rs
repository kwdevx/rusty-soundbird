use crate::{Context, Error};

use super::join::handle_join;

#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn skip(
    ctx: Context<'_>, // #[description = "Url to the song"] url: String,
) -> Result<(), Error> {
    ctx.defer().await?;

    handle_skip_current_song(ctx, 0, 2).await?;

    Ok(())
}

async fn handle_skip_current_song(
    ctx: Context<'_>,
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

    let manager = songbird::get(ser_ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    match manager.get(guild_id) {
        Some(handler_lock) => {
            let handler = handler_lock.lock().await;

            let queue = handler.queue();
            let _ = queue.skip();

            // handle metadata for spotify adaptor
            ctx.reply(format!("Song skipped: {} in queue.", queue.len() - 1))
                .await?;
        }
        _ => {
            ctx.reply("Not in a voice channel to play in, joining...")
                .await?;
            if let Ok(_) = handle_join(ctx).await {
                let future = Box::pin(handle_skip_current_song(
                    ctx,
                    trial_time + 1,
                    max_trial_time,
                ));
                future.await?;
            }
        }
    }
    Ok(())
}
