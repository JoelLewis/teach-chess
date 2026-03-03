use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillRating {
    pub id: String,
    pub player_id: String,
    pub category: String,
    pub rating: f64,
    pub rd: f64,
    pub volatility: f64,
    pub games_count: u32,
    pub last_updated: String,
}

impl SkillRating {
    pub fn default_for(player_id: &str, category: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            player_id: player_id.to_string(),
            category: category.to_string(),
            rating: 1200.0,
            rd: 350.0,
            volatility: 0.06,
            games_count: 0,
            last_updated: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillProfile {
    pub ratings: Vec<SkillRating>,
    pub overall_rating: f64,
    pub strongest_category: Option<String>,
    pub weakest_category: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DifficultyTarget {
    pub target_rating: f64,
    pub min_rating: u32,
    pub max_rating: u32,
}
