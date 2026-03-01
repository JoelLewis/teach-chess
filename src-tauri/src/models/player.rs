use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Player {
    pub id: String,
    pub display_name: String,
    pub created_at: String,
    pub games_played: u32,
    pub settings: PlayerSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerSettings {
    /// Default engine strength preset
    #[serde(default = "default_engine_elo")]
    pub preferred_engine_elo: u32,

    /// Board orientation preference
    #[serde(default = "default_true")]
    pub auto_flip_board: bool,

    /// Show evaluation bar during play
    #[serde(default = "default_true")]
    pub show_eval_bar: bool,

    /// Analysis depth for post-game review
    #[serde(default = "default_review_depth")]
    pub review_depth: u32,
}

fn default_engine_elo() -> u32 {
    1350
}
fn default_true() -> bool {
    true
}
fn default_review_depth() -> u32 {
    18
}

impl Default for PlayerSettings {
    fn default() -> Self {
        Self {
            preferred_engine_elo: default_engine_elo(),
            auto_flip_board: true,
            show_eval_bar: true,
            review_depth: default_review_depth(),
        }
    }
}
