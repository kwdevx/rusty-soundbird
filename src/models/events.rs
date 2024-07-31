use std::sync::Arc;

use poise::serenity_prelude::{async_trait, Http};
use songbird::id::ChannelId;
use songbird::{Event, EventContext, EventHandler as VoiceEventHandler};

struct SongEndNotifier {
    chan_id: ChannelId,
    http: Arc<Http>,
}

#[async_trait]
impl VoiceEventHandler for SongEndNotifier {
    async fn act(&self, _ctx: &EventContext<'_>) -> Option<Event> {
        // check_msg(
        // self.chan_id
        //     .say(&self.http, "Song faded out completely!")
        //     .await,
        // );

        None
    }
}
