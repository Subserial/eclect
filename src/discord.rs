use discord_rich_presence::activity::{Activity, ActivityType};
use discord_rich_presence::{DiscordIpc, activity};

pub fn activate(
    app_id: &str,
) -> Result<discord_rich_presence::DiscordIpcClient, Box<dyn std::error::Error>> {
    let mut client = discord_rich_presence::DiscordIpcClient::new(app_id)?;
    client.connect()?;
    Ok(client)
}

fn build_assets<'a>(text: &'a str, images: &'a Vec<prost_lastfm::Image>) -> activity::Assets<'a> {
    match images.iter().find(|image| image.size == "medium") {
        Some(image) => activity::Assets::new()
            .large_image(&image.text)
            .large_text(&text),
        None => activity::Assets::new(),
    }
}

pub fn set_track(
    client: &mut discord_rich_presence::DiscordIpcClient,
    track: Option<prost_lastfm::Track>,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let Some(track) = track else {
        client.clear_activity()?;
        return Ok(None);
    };
    let title = track.name.as_str();
    let album = track
        .album
        .as_ref()
        .map(|a| a.text.as_str())
        .unwrap_or("Unknown Album");
    let artist = track
        .artist
        .as_ref()
        .map(|a| a.text.as_str())
        .unwrap_or("Unknown Artist");
    let page = track.url.as_str();

    let hover_text = format!("{} - {}", artist, album);

    client.set_activity(
        Activity::new()
            .activity_type(ActivityType::Listening)
            .details(&title)
            .assets(build_assets(&hover_text, &track.image))
            .buttons(vec![activity::Button::new("View on last.fm", page)]),
    )?;

    Ok(Some(format!("{} - {}", artist, title)))
}
