use crate::{commands::help, Context, Error};
use rand::Rng;

const FRUIT: &[&str] = &["🍎", "🍌", "🍊", "🍉", "🍇", "🍓"];

/// Respond with a random fruit
///
/// Subcommands can be used to get a specific fruit
#[poise::command(
    slash_command,
    prefix_command,
    subcommands("apple", "help"),
    category = "Vegan"
)]
pub async fn fruit(ctx: Context<'_>) -> Result<(), Error> {
    let response = FRUIT[rand::thread_rng().gen_range(0..FRUIT.len())];
    ctx.say(response).await?;
    Ok(())
}
/// Respond with an apple
#[poise::command(slash_command, prefix_command, subcommands("red"))]
pub async fn apple(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("🍎").await?;
    Ok(())
}

/// Respond with a red apple
#[poise::command(slash_command, prefix_command)]
async fn red(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("🍎").await?;
    Ok(())
}
