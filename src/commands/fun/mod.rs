use crate::Context;
use crate::Error;
use rand::{thread_rng, Rng};

#[poise::command(prefix_command)]
pub async fn roll(
    ctx: Context<'_>,
    #[description = "first limit"] range1: Option<u32>,
    #[description = "second limit"] range2: Option<u32>,
) -> Result<(), Error> {
    if range2.is_none() && range1.is_none() {
        let x = thread_rng().gen_range(1..=6u32).to_string();
        ctx.say(x).await?;
    } else if range2.is_none() && range1.is_some() {
        let x = thread_rng().gen_range(1..=range1.unwrap()).to_string();
        ctx.say(x).await?;
    } else if range2.is_some() && range1.is_some() {
        if range2.unwrap() < range1.unwrap() {
            ctx.say("First value is greater than the second!").await?;
            return Ok(());
        } else {
            let x = thread_rng()
                .gen_range(range1.unwrap()..=range2.unwrap())
                .to_string();
            ctx.say(x).await?;
        }
    } else {
        ctx.say("Something went wrong!").await?;
    }
    Ok(())
}
