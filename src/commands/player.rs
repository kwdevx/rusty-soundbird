use crate::{Context, Error, HttpKey};
use poise::serenity_prelude::async_trait;
use songbird::{
    events::{Event, EventContext, EventHandler as VoiceEventHandler, TrackEvent},
    input::YoutubeDl,
};

struct TrackErrorNotifier;

#[async_trait]
impl VoiceEventHandler for TrackErrorNotifier {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        if let EventContext::Track(track_list) = ctx {
            for (state, handle) in *track_list {
                println!(
                    "Track {:?} encountered an error: {:?}",
                    handle.uuid(),
                    state.playing
                );
            }
        }

        None
    }
}

#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn play(
    ctx: Context<'_>,
    #[description = "Message to echo (enter a link or ID)"] url: String,
) -> Result<(), Error> {
    ctx.defer().await?;

    let do_search = !url.starts_with("http");

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

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;

        let src = if do_search {
            YoutubeDl::new_search(http_client, url)
        } else {
            YoutubeDl::new(http_client, url)
        };

        let _ = handler.play_input(src.clone().into());

        ctx.say("Playing song").await?;
    } else {
        ctx.say("Not in a voice channel to play in").await?;
    }

    Ok(())
}

#[poise::command(slash_command)]
pub async fn join(
    ctx: Context<'_>,
    // #[description = "Message to echo (enter a link or ID)"] url: String,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    let author_id = ctx.author().id;
    let ser_ctx = ctx.serenity_context();

    let (guild_id, channel_id) = {
        let guild = ctx.clone().guild().expect("have guild");
        let channel_id = guild
            .voice_states
            .get(&author_id)
            .and_then(|voice_state| voice_state.channel_id);

        (guild.id, channel_id)
    };

    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            ctx.say("Not in a voice channel").await?;
            return Ok(());
        }
    };

    let manager = songbird::get(ser_ctx).await.expect("have manager").clone();

    if let Ok(handler_lock) = manager.join(guild_id, connect_to).await {
        // Attach an event handler to see notifications of all track errors.
        let mut handler = handler_lock.lock().await;
        handler.add_global_event(TrackEvent::Error.into(), TrackErrorNotifier);
    }

    Ok(())
}
