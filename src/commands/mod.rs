pub mod help;
pub mod ping;
pub mod player;

use help::help;
use ping::ping;
use player::{join::join, play::play, query::query, queue::queue};

use crate::Error;
use poise::Command;

pub fn create_command() -> Vec<Command<crate::Data, Error>> {
    vec![help(), ping(), join(), play(), query(), queue()]
}
