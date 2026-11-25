use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Team {
    pub id: String,
    pub abbreviation: String,
    pub display_name: String,
    pub short_name: String,
    pub logo: String,
    pub color: String,
    pub alternate_color: String,
}

#[derive(Debug, Clone)]
pub struct Game {
    pub id: String,
    pub home: Team,
    pub away: Team,
    pub home_score: i32,
    pub away_score: i32,
    pub status: String,
    pub detail: String,
    pub is_live: bool,
    pub start_time: String,
}

#[derive(Deserialize)]
struct EspnScoreboardResponse {
    #[serde(default)]
    events: Vec<EspnEvent>,
}

#[derive(Deserialize)]
struct EspnEvent {
    id: String,
    #[serde(default)]
    competitions: Vec<EspnCompetition>,
    date: String,
    #[serde(default)]
    status: Option<EspnStatus>,
}

#[derive(Deserialize)]
struct EspnCompetition {
    #[serde(default)]
    competitors: Vec<EspnCompetitor>,
}

#[derive(Deserialize)]
struct EspnCompetitor {
    #[serde(rename = "homeAway", default)]
    home_away: String,
    #[serde(default)]
    score: String,
    team: EspnTeamRef,
}

#[derive(Deserialize, Clone)]
struct EspnTeamRef {
    id: String,
    #[serde(default)]
    abbreviation: String,
    #[serde(rename = "displayName", default)]
    display_name: String,
    #[serde(rename = "shortDisplayName", default)]
    short_display_name: String,
    #[serde(default)]
    logos: Vec<EspnLogo>,
    #[serde(default)]
    color: String,
    #[serde(rename = "alternateColor", default)]
    alternate_color: String,
}

impl EspnTeamRef {
    fn into_team(self) -> Team {
        let logo = self
            .logos
            .first()
            .map(|l| l.href.clone())
            .filter(|href| !href.is_empty())
            .unwrap_or_else(|| {
                format!(
                    "https://a.espncdn.com/i/teamlogos/nba/500/{}.png",
                    self.abbreviation.to_lowercase()
                )
            });

        Team {
            id: self.id,
            abbreviation: self.abbreviation,
            display_name: self.display_name,
            short_name: self.short_display_name,
            logo,
            color: self.color,
            alternate_color: self.alternate_color,
        }
    }
}

#[derive(Deserialize, Clone)]
struct EspnLogo {
    href: String,
}

#[derive(Deserialize)]
struct EspnStatus {
    #[serde(rename = "type")]
    status_type: EspnStatusType,
}

#[derive(Deserialize)]
struct EspnStatusType {
    #[serde(default)]
    detail: String,
    #[serde(rename = "shortDetail", default)]
    short_detail: String,
    #[serde(default)]
    state: String,
}

#[derive(Deserialize)]
struct EspnTeamsResponse {
    #[serde(default)]
    sports: Vec<EspnSport>,
}

#[derive(Deserialize)]
struct EspnSport {
    #[serde(default)]
    leagues: Vec<EspnLeague>,
}

#[derive(Deserialize)]
struct EspnLeague {
    #[serde(default)]
    teams: Vec<EspnTeamWrapper>,
}

#[derive(Deserialize)]
struct EspnTeamWrapper {
    team: EspnTeamRef,
}

pub struct EspnClient {
    client: Client,
    base_url: String,
}

impl EspnClient {
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(12))
            .user_agent("NBA-Tracker/0.1.0")
            .build()?;

        Ok(Self {
            client,
            base_url: "https://site.api.espn.com/apis/site/v2/sports/basketball/nba".to_string(),
        })
    }

    pub async fn fetch_scoreboard(&self, team_ids: &[String]) -> Result<Vec<Game>> {
        let url = format!("{}/scoreboard", self.base_url);
        let response: EspnScoreboardResponse = self.client.get(&url).send().await?.json().await?;

        let mut games = Vec::new();

        for event in response.events {
            let Some(comp) = event.competitions.first() else {
                continue;
            };
            if comp.competitors.len() != 2 {
                continue;
            }

            let home = comp.competitors.iter().find(|c| c.home_away == "home");
            let away = comp.competitors.iter().find(|c| c.home_away == "away");
            let (Some(home), Some(away)) = (home, away) else {
                continue;
            };

            let home_team = home.team.clone().into_team();
            let away_team = away.team.clone().into_team();

            if !team_ids.contains(&home_team.id) && !team_ids.contains(&away_team.id) {
                continue;
            }

            let (status, detail, is_live) = match &event.status {
                Some(status) => {
                    let ty = &status.status_type;
                    let status_text = if ty.state == "post" && ty.detail.is_empty() {
                        "Final".to_string()
                    } else if ty.detail.is_empty() {
                        ty.short_detail.clone()
                    } else {
                        ty.detail.clone()
                    };
                    (status_text, ty.short_detail.clone(), ty.state == "in")
                }
                None => (String::new(), String::new(), false),
            };

            games.push(Game {
                id: event.id,
                home_score: home.score.parse().unwrap_or(0),
                away_score: away.score.parse().unwrap_or(0),
                home: home_team,
                away: away_team,
                status,
                detail,
                is_live,
                start_time: event.date.clone(),
            });
        }

        Ok(games)
    }

    pub async fn search_teams(&self, query: &str) -> Result<Vec<Team>> {
        let q = query.trim().to_lowercase();
        if q.is_empty() {
            return Ok(Vec::new());
        }

        let mut teams = self.all_teams().await?;
        teams.retain(|t| {
            let haystack = format!(
                "{} {} {}",
                t.display_name.to_lowercase(),
                t.abbreviation.to_lowercase(),
                t.short_name.to_lowercase()
            );
            haystack.contains(&q)
        });

        Ok(teams)
    }

    pub async fn all_teams(&self) -> Result<Vec<Team>> {
        let url = format!("{}/teams", self.base_url);
        let response: EspnTeamsResponse = self.client.get(&url).send().await?.json().await?;

        let mut teams = Vec::new();
        for sport in response.sports {
            for league in sport.leagues {
                for wrapper in league.teams {
                    teams.push(wrapper.team.into_team());
                }
            }
        }

        teams.sort_by(|a, b| a.display_name.cmp(&b.display_name));
        teams.dedup_by(|a, b| a.id == b.id);
        Ok(teams)
    }
}
