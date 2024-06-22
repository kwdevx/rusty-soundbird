pub mod fruit;
pub mod help;
pub mod ping;
pub mod player;

use fruit::fruit;
use help::help;
use ping::ping;
use player::join;
use player::play;

use crate::Error;
use poise::Command;

pub fn create_command() -> Vec<Command<crate::Data, Error>> {
    vec![help(), ping(), fruit(), join(), play()]
}
