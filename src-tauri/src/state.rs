use anyhow::Result;
use chrono::Local;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, Runtime};

use crate::espn::{EspnClient, Team};
use crate::storage::Storage;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FollowedTeam {
    pub id: String,
    pub abbreviation: String,
    pub display_name: String,
    pub logo: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GameDisplay {
    pub id: String,
    pub home_team: FollowedTeam,
    pub away_team: FollowedTeam,
    pub home_score: i32,
    pub away_score: i32,
    pub status: String,
    pub detail: String,
    pub is_live: bool,
    pub start_time: String,
    pub game_url: String,
}

pub type SharedState = std::sync::Arc<AppState>;

pub struct AppState {
    storage: Storage,
    espn: EspnClient,
    teams: std::sync::Mutex<Vec<FollowedTeam>>,
    games: std::sync::Mutex<Vec<GameDisplay>>,
    last_updated: std::sync::Mutex<Option<String>>,
}

impl AppState {
    pub fn new() -> Result<Self> {
        let storage = Storage::new()?;
        let teams = storage.load_teams()?;
        Ok(Self {
            storage,
            espn: EspnClient::new()?,
            teams: std::sync::Mutex::new(teams),
            games: std::sync::Mutex::new(Vec::new()),
            last_updated: std::sync::Mutex::new(None),
        })
    }

    pub fn teams(&self) -> Vec<FollowedTeam> {
        self.teams.lock().unwrap().clone()
    }

    pub fn games(&self) -> Vec<GameDisplay> {
        self.games.lock().unwrap().clone()
    }

    pub fn last_updated(&self) -> Option<String> {
        self.last_updated.lock().unwrap().clone()
    }

    pub fn has_live(&self) -> bool {
        self.games.lock().unwrap().iter().any(|g| g.is_live)
    }

    pub fn add_team(&self, team: FollowedTeam) -> Result<()> {
        let mut teams = self.teams.lock().unwrap();
        if !teams.iter().any(|t| t.id == team.id) {
            teams.push(team);
            self.storage.save_teams(&teams)?;
        }
        Ok(())
    }

    pub fn remove_team(&self, team_id: &str) -> Result<()> {
        let mut teams = self.teams.lock().unwrap();
        teams.retain(|t| t.id != team_id);
        self.storage.save_teams(&teams)?;
        Ok(())
    }

    pub async fn search_teams(&self, query: &str) -> Result<Vec<Team>> {
        self.espn.search_teams(query).await
    }

    pub async fn refresh(&self) -> Result<()> {
        let teams = self.teams();
        if teams.is_empty() {
            *self.games.lock().unwrap() = Vec::new();
            *self.last_updated.lock().unwrap() = Some(stamp());
            return Ok(());
        }

        let all_ids: Vec<String> = teams.iter().map(|t| t.id.clone()).collect();
        let games = self.espn.fetch_scoreboard(&all_ids).await?;

        let mut display = Vec::new();
        for game in games {
            let followed = |team: &crate::espn::Team| FollowedTeam {
                id: team.id.clone(),
                abbreviation: team.abbreviation.clone(),
                display_name: team.display_name.clone(),
                logo: team.logo.clone(),
            };

            display.push(GameDisplay {
                id: game.id.clone(),
                home_team: followed(&game.home),
                away_team: followed(&game.away),
                home_score: game.home_score,
                away_score: game.away_score,
                status: game.status,
                detail: game.detail,
                is_live: game.is_live,
                start_time: game.start_time,
                game_url: format!("https://www.espn.com/nba/game/_/gameId/{}", game.id),
            });
        }

        // live games first, then the rest by start time
        display.sort_by_key(|g| (!g.is_live, g.start_time.clone()));

        *self.games.lock().unwrap() = display;
        *self.last_updated.lock().unwrap() = Some(stamp());
        Ok(())
    }
}

fn stamp() -> String {
    Local::now().format("%-I:%M %p").to_string()
}

pub fn format_start_time(raw: &str) -> String {
    match chrono::DateTime::parse_from_rfc3339(raw) {
        Ok(dt) => {
            let local = dt.with_timezone(&Local);
            if local.date_naive() == Local::now().date_naive() {
                local.format("%-I:%M %p").to_string()
            } else {
                local.format("%a %-I:%M %p").to_string()
            }
        }
        Err(_) => raw.to_string(),
    }
}

pub fn shared<R: Runtime>(app: &AppHandle<R>) -> SharedState {
    app.state::<SharedState>().inner().clone()
}
