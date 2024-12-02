use crate::{Context, Error, HttpKey};
use songbird::input::YoutubeDl;

#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn queue(
    ctx: Context<'_>,
    #[description = "Url to the song"] url: String,
) -> Result<(), Error> {
    ctx.defer().await?;

    if !url.starts_with("http") {
        ctx.reply("Must provide a valid URL").await?;
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

            let src = YoutubeDl::new(http_client, url);

            let q_len = handler.queue().len();
            println!("current queue length {}", q_len);

            handler.enqueue_input(src.into()).await;

            ctx.reply("Queued song").await?;
        }
        None => {
            ctx.reply("Not in a voice channel to play in").await?;
        }
    }

    Ok(())
}
