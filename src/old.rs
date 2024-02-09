mod structs;
mod app;
mod ui;

use clap::Parser;
use reqwest;
use serde::{Deserialize, Serialize};
use tokio;
use std::io::{self, stdout};

use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{prelude::*, widgets::*};

/// Simple program to retrieve nba game data
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The date the shown results
    #[arg(short, long)]
    date: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct GameData {
    data: Vec<Game>,
    meta: MetaData,
}

#[derive(Serialize, Deserialize, Debug)]
struct Game {
    id: u32,
    date: String,
    home_team: Team,
    home_team_score: u32,
    period: u8,
    postseason: bool,
    season: u32,
    status: String,
    time: String,
    visitor_team: Team,
    visitor_team_score: u32,
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
struct MetaData {
    current_page: u32,
    next_page: Option<u32>,
    per_page: u32,
}

fn main() {

    println!("These are the results for {}!", args.date);

    nba_client(args.date);
}

#[tokio::main]
async fn nba_client(date: String) -> Result<(), reqwest::Error> {
    let client = reqwest::Client::new();

    let query = format!("?dates[]={}", date);

    // Build the request with the query parameters
    let response = client
        .get(format!("{}{}", "https://www.balldontlie.io/api/v1/games/", query))
        .send()
        .await?;

    // Check if the request was successful
    if response.status().is_success() {
        // Parse the response body as JSON, String, etc.
        let json_response = response.text().await?;

        let game_data = parse_json(json_response);

        print_game_data(game_data);
    } else {
        // Handle the error
        eprintln!("Request failed with status: {}", response.status());
    }

    Ok(())
}

fn parse_json(json_data: String) -> GameData {
    let result: Result<GameData, serde_json::Error> = serde_json::from_str(&*json_data);

    match result {
        Ok(game_data) => {
            game_data
        }
        Err(e) => {
            panic!("Error parsing JSON: {:?}", e)
        }
    }
}

fn print_game_data(game_data: GameData) {
    for game in game_data.data {
        let line = format!("{} {}:{} {}", game.home_team.abbreviation, game.home_team_score, game.visitor_team_score, game.visitor_team.abbreviation);

        println!("{}", line);
    }
}