use std::collections::HashMap;

use crate::{Context, Error};
use poise::serenity_prelude::{async_trait, UserId, VoiceState};
use songbird::{
    events::{Event, EventContext, EventHandler as VoiceEventHandler},
    TrackEvent,
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

#[poise::command(slash_command)]
pub async fn join(
    ctx: Context<'_>,
    // #[description = "Message to echo (enter a link or ID)"] url: String,
) -> Result<(), Error> {
    ctx.defer().await?;

    handle_join(ctx).await?;

    Ok(())
}

pub async fn handle_join(ctx: Context<'_>) -> Result<(), Error> {
    let author_id = ctx.author().id;
    let ser_ctx = ctx.serenity_context();

    let (guild_id, channel_id) = {
        let guild = ctx.clone().guild().expect("have guild");

        let voice_status: &HashMap<UserId, VoiceState> = &guild.voice_states;
        let voice_state = voice_status.get(&author_id);
        let channel_id = voice_state.and_then(|voice_state| voice_state.channel_id);

        (guild.id, channel_id)
    };

    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            ctx.reply("Not in a voice channel").await?;
            return Ok(());
        }
    };

    let manager = songbird::get(ser_ctx).await.expect("have manager");

    match manager.join(guild_id, connect_to).await {
        Ok(_) => {
            // Attach an event handler to see notifications of all track errors.

            ctx.reply("Joined").await?;
        }
        Err(e) => {
            println!("Join error {e:?}");
            ctx.reply("Cannot join channel").await?;
        }
    }

    Ok(())
}
