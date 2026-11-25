use tauri::menu::{Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::tray::{TrayIcon, TrayIconBuilder};
use tauri::{AppHandle, Runtime};

use crate::state::{format_start_time, shared, AppState, GameDisplay};

fn disabled<R: Runtime>(
    app: &AppHandle<R>,
    id: &str,
    text: impl AsRef<str>,
) -> tauri::Result<MenuItem<R>> {
    MenuItem::with_id(app, id, text, false, None::<&str>)
}

fn is_final(game: &GameDisplay) -> bool {
    game.status.to_lowercase().contains("final")
}

fn game_line(game: &GameDisplay) -> String {
    if game.is_live {
        format!(
            "{} {} - {} {}   {}",
            game.away_team.abbreviation,
            game.away_score,
            game.home_team.abbreviation,
            game.home_score,
            game.detail
        )
    } else if is_final(game) {
        format!(
            "{} {} - {} {}   Final",
            game.away_team.abbreviation,
            game.away_score,
            game.home_team.abbreviation,
            game.home_score
        )
    } else {
        format!(
            "{} @ {}   {}",
            game.away_team.abbreviation,
            game.home_team.abbreviation,
            format_start_time(&game.start_time)
        )
    }
}

pub fn build_menu<R: Runtime>(app: &AppHandle<R>, state: &AppState) -> tauri::Result<Menu<R>> {
    let menu = Menu::new(app)?;
    let games = state.games();
    let teams = state.teams();

    if teams.is_empty() {
        menu.append(&disabled(app, "empty_teams", "No teams followed yet")?)?;
    } else if games.is_empty() {
        menu.append(&disabled(app, "empty_games", "No games today")?)?;
    } else {
        let live: Vec<&GameDisplay> = games.iter().filter(|g| g.is_live).collect();
        let finals: Vec<&GameDisplay> =
            games.iter().filter(|g| !g.is_live && is_final(g)).collect();
        let upcoming: Vec<&GameDisplay> =
            games.iter().filter(|g| !g.is_live && !is_final(g)).collect();

        let mut first = true;

        if !live.is_empty() {
            menu.append(&disabled(app, "live_header", "LIVE NOW")?)?;
            for game in &live {
                menu.append(&MenuItem::with_id(
                    app,
                    format!("game_{}", game.id),
                    game_line(game),
                    true,
                    None::<&str>,
                )?)?;
            }
            first = false;
        }

        if !finals.is_empty() {
            if !first {
                menu.append(&PredefinedMenuItem::separator(app)?)?;
            }
            menu.append(&disabled(app, "final_header", "FINAL")?)?;
            for game in &finals {
                menu.append(&MenuItem::with_id(
                    app,
                    format!("game_{}", game.id),
                    game_line(game),
                    true,
                    None::<&str>,
                )?)?;
            }
            first = false;
        }

        if !upcoming.is_empty() {
            if !first {
                menu.append(&PredefinedMenuItem::separator(app)?)?;
            }
            menu.append(&disabled(app, "upcoming_header", "TODAY")?)?;
            for game in &upcoming {
                menu.append(&MenuItem::with_id(
                    app,
                    format!("game_{}", game.id),
                    game_line(game),
                    true,
                    None::<&str>,
                )?)?;
            }
        }
    }

    menu.append(&PredefinedMenuItem::separator(app)?)?;

    let mut team_items: Vec<MenuItem<R>> = Vec::new();
    for team in &teams {
        team_items.push(MenuItem::with_id(
            app,
            format!("team_{}", team.id),
            format!("{} ({})", team.display_name, team.abbreviation),
            true,
            None::<&str>,
        )?);
    }
    if team_items.is_empty() {
        team_items.push(disabled(app, "no_team_items", "None yet")?);
    }
    let team_refs: Vec<&dyn tauri::menu::IsMenuItem<R>> = team_items
        .iter()
        .map(|i| i as &dyn tauri::menu::IsMenuItem<R>)
        .collect();
    let teams_menu = Submenu::with_items(app, "Followed Teams", true, &team_refs)?;
    menu.append(&teams_menu)?;

    menu.append(&MenuItem::with_id(app, "refresh", "Refresh Now", true, None::<&str>)?)?;
    menu.append(&MenuItem::with_id(app, "settings", "Settings...", true, None::<&str>)?)?;

    if let Some(updated) = state.last_updated() {
        menu.append(&PredefinedMenuItem::separator(app)?)?;
        menu.append(&disabled(app, "updated", format!("Updated {}", updated))?)?;
    }

    menu.append(&PredefinedMenuItem::separator(app)?)?;
    menu.append(&MenuItem::with_id(app, "quit", "Quit NBA Tracker", true, None::<&str>)?)?;

    Ok(menu)
}

pub fn build<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<TrayIcon<R>> {
    let state = shared(app);
    let menu = build_menu(app, &state)?;

    let builder = TrayIconBuilder::with_id("main")
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(on_menu_event);

    let builder = match tauri::image::Image::from_bytes(include_bytes!("../icons/tray.png")) {
        Ok(icon) => builder.icon(icon),
        Err(_) => match app.default_window_icon().cloned() {
            Some(icon) => builder.icon(icon),
            None => builder,
        },
    };

    #[cfg(target_os = "macos")]
    let builder = builder.icon_as_template(true);

    builder.build(app)
}

pub fn rebuild<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let state = shared(app);
    let menu = build_menu(app, &state)?;
    if let Some(tray) = app.tray_by_id("main") {
        tray.set_menu(Some(menu))?;
    }
    Ok(())
}

fn on_menu_event<R: Runtime>(app: &AppHandle<R>, event: tauri::menu::MenuEvent) {
    let id = event.id().as_ref();

    match id {
        "quit" => app.exit(0),
        "settings" => crate::show_settings(app),
        "refresh" => crate::spawn_refresh(app),
        _ => {
            if let Some(game_id) = id.strip_prefix("game_") {
                let state = shared(app);
                if let Some(game) = state.games().into_iter().find(|g| g.id == game_id) {
                    open(&game.game_url);
                }
            } else if let Some(team_id) = id.strip_prefix("team_") {
                let state = shared(app);
                if let Some(team) = state.teams().into_iter().find(|t| t.id == team_id) {
                    let url = format!(
                        "https://www.espn.com/nba/team/_/name/{}",
                        team.abbreviation.to_lowercase()
                    );
                    open(&url);
                }
            }
        }
    }
}

fn open(url: &str) {
    if let Err(e) = tauri_plugin_opener::open_url(url, None::<&str>) {
        eprintln!("failed to open {}: {}", url, e);
    }
}
