use serde::{Serialize, Deserialize};

// TODO: more fields

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GameManagerDB {
    bg_volume: f32,
    sfx_volume: f32,
    // #[serde(rename = "playerUUID")]
    // player_uuid: String,
    player_name: String,
    #[serde(rename = "playerUserID")]
    player_user_id: i32,
    player_frame: i32,
    player_ship: i32,
    player_ball: i32,
    player_bird: i32,
    player_dart: i32,
    player_robot: i32,
    player_spider: i32,
    player_color: i32,
    #[serde(rename = "playerColor2")]
    player_color_secondary: i32,
    player_streak: i32,
    player_death_effect: i32,
    player_icon_type: i32,
    #[serde(default)]
    player_glow: bool,
    #[serde(default)]
    secret_number: i32,
    // hasRP: bool,
    // valueKeeper,
    // unlockValueKeeper,
    // customObjectDict,
    // reportedAchievements,
    #[serde(default)]
    show_song_markers: bool,
    #[serde(default)]
    show_progress_bar: bool,
    #[serde(default)]
    clicked_garage: bool,
    #[serde(default)]
    clicked_editor: bool,
    #[serde(default)]
    clicked_practice: bool,
    #[serde(default)]
    showed_editor_guide: bool,
    #[serde(default)]
    showed_low_detail_dialog: bool,
    bootups: i32,
    #[serde(default)]
    has_rated_game: bool,
    binary_version: i32,
    resolution: i32,
    tex_quality: i32
}
