use anyhow::Result;
use crossterm::{
    event::{self, Event::Key, KeyCode::Char},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    prelude::{CrosstermBackend, Terminal, Frame},
    widgets::Paragraph,
};
use ratatui::widgets::{Block, Borders};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

fn startup() -> Result<()> {
    enable_raw_mode()?;
    execute!(std::io::stderr(), EnterAlternateScreen)?;
    Ok(())
}

fn shutdown() -> Result<()> {
    execute!(std::io::stderr(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

// App state
struct App {
    day: DateTime<Utc>,
    should_quit: bool,
    game_data: Option<GameData>,
}

// App ui render function
fn ui(app: &App, f: &mut Frame) {
    let date = app.day.format("%Y-%m-%d").to_string();

    if app.game_data.is_some() {
        let game_data = &app.game_data.as_ref().unwrap().data;

        let mut text = String::new();

        for game in game_data {
            let line = format!("{} {}:{} {}\n", game.home_team.abbreviation, game.home_team_score, game.visitor_team_score, game.visitor_team.abbreviation);
            text.push_str(&line);
        }

        text.push_str("\nNavigation:\n");
        text.push_str("one day: j|k\n");
        text.push_str("one week: h|l\n");
        text.push_str("today: t\n");
        text.push_str("quit: q");

        if app.day <= Utc::now() {
            f.render_widget(Paragraph::new(text).block(Block::default().title(format!("NBA Game results of: {}", date)).borders(Borders::ALL)), f.size());
        } else {
            f.render_widget(Paragraph::new("").block(Block::default().title(format!("{} is in the future.", date)).borders(Borders::ALL)), f.size());
        }
    }
}

// App update function
fn update(app: &mut App) -> Result<()> {
    if event::poll(std::time::Duration::from_millis(250))? {
        if let Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press {
                match key.code {
                    Char('h') => app.day += Duration::days(7),
                    Char('j') => app.day += Duration::days(1),
                    Char('k') => app.day -= Duration::days(1),
                    Char('l') => app.day -= Duration::days(7),
                    Char('t') => app.day = Utc::now(),
                    Char('q') => app.should_quit = true,
                    _ => {}
                }
                app.game_data = get_nba_data(app.day)
            }
        }
    }
    Ok(())
}

fn run() -> Result<()> {
    // ratatui terminal
    let mut t = Terminal::new(CrosstermBackend::new(std::io::stderr()))?;

    // application state
    let mut app = App { day: Utc::now(), should_quit: false, game_data: get_nba_data(Utc::now() - Duration::days(1)) };

    loop {
        // application update
        update(&mut app)?;

        // application render
        t.draw(|f| {
            ui(&app, f);
        })?;

        // application exit
        if app.should_quit {
            break;
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    // setup terminal
    startup()?;

    let result = run();

    // teardown terminal before unwrapping Result of app run
    shutdown()?;

    result?;

    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
struct Team {
    id: u32,
    abbreviation: String,
    city: String,
    conference: String,
    division: String,
    full_name: String,
    name: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Game {
    id: u32,
    date: String,
    home_team: Team,
    home_team_score: u32,
    period: u32,
    postseason: bool,
    season: u32,
    status: String,
    time: Option<String>,
    visitor_team: Team,
    visitor_team_score: u32,
}

#[derive(Serialize, Deserialize, Debug)]
struct Meta {
    current_page: u32,
    next_page: Option<u32>,
    per_page: u32,
}

#[derive(Serialize, Deserialize, Debug)]
struct GameData {
    data: Vec<Game>,
    meta: Meta,
}


fn get_nba_data(date_time: DateTime<Utc>) -> Option<GameData> {
    let client = reqwest::blocking::Client::new();

    let date = date_time.format("%Y-%m-%d").to_string();

    let query = format!("?dates[]={}", date);

    // Build the request with the query parameters
    let response = client
        .get(format!("{}{}", "https://www.balldontlie.io/api/v1/games/", query))
        .send();


    // Parse the response body as JSON, String, etc.
    let json_response = response.expect("Could not read data").text().ok()?;

    let game_data = parse_json(json_response);

    Some(game_data)
}

fn parse_json(json_data: String) -> GameData {
    let result: Result<GameData, serde_json::Error> = serde_json::from_str(&json_data);

    match result {
        Ok(game_data) => {
            game_data
        }
        Err(e) => {
            panic!("Error parsing JSON: {:?}", e)
        }
    }
}