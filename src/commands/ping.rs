use crate::{Context, Error};

#[poise::command(slash_command, prefix_command, category = "util")]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("Pong!").await?;
    Ok(())
}
