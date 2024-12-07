pub mod help;
pub mod ping;
pub mod player;

use help::help;
use ping::ping;
use player::{
    join::join, query::query, queue::queue, skip::skip, spotify::spotify, stop::stop, yt::yt,
};

use crate::Error;
use poise::Command;

pub fn create_command() -> Vec<Command<crate::Data, Error>> {
    vec![
        help(),
        ping(),
        join(),
        yt(),
        spotify(),
        query(),
        queue(),
        skip(),
        stop(),
    ]
}
