use std::time::Duration;

use serenity::builder::CreateEmbed;

pub mod join;
pub mod leave;
pub mod now_playing;
pub mod pause;
pub mod play;
pub mod play_playlist;
pub mod queue;
pub mod resume;
pub mod seek;
pub mod skip;
pub mod stop;

fn yt_embed<'a>(
    e: &'a mut CreateEmbed,
    track_info: &lavalink_rs::model::Info,
    queue_len: usize,
    requester: &str,
) -> &'a mut CreateEmbed {
    e.title(&track_info.title);
    e.url(&track_info.uri);
    e.field("Position in queue", queue_len, true);
    e.field(
        "Song Duration",
        {
            let duration = Duration::from_millis(track_info.length).as_secs();
            let seconds = duration % 60;
            let minutes = (duration / 60) % 60;
            let hours = (duration / 60) / 60;
            format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
        },
        true,
    );
    e.thumbnail(format!(
        "https://img.youtube.com/vi/{}/maxresdefault.jpg",
        track_info.identifier
    ));
    e.footer(|f| f.text(format!("Submited by {}", requester)))
}
