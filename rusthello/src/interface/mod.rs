// This module provides interface functionalities and manages all the input/output part of the program

use opponents;
use std::cmp::Ordering;
use std::io::{self, Write};
use rusthello_lib::game;

pub mod human;

pub fn start_game() -> (opponents::Opponent, opponents::Opponent) {

    // Print a start message
    println!("\n\n\n\t         RUSThello");
    println!("\t           =====");
    println!("\t   a simple Reversi game");
    println!("\t written in Rust with love");
    println!("\t          v 0.2.1");

    println!("Players:");
    println!("1) Human");
    println!("2) External AI");

    let light = select_opponent("Light");
    let dark = select_opponent("Dark");

    (light, dark)
}


fn select_opponent(name: &str) -> opponents::Opponent {

    print!("Select {} player: ", name);

    loop {

        let mut input = String::new();

        // Read the input
        let _ = io::stdout().flush();
        io::stdin().read_line(&mut input)
            .ok()
            .expect("Failed to read line");

        let input: u32 = match input.trim().parse() {
            Ok(num) => num,
            Err(_) => continue,
        };

        match input {
            1 => return opponents::Opponent::Human,
            2 => {
                print!("External AI's full path: ");

                let mut input = String::new();
                let _ = io::stdout().flush();
                io::stdin().read_line(&mut input)
                    .ok()
                    .expect("Failed to read line");

                return opponents::Opponent::ExternalAI { ai_path: input.trim().to_string() };
            }
            _ => {
                println!("Invalid input. Try again");
                continue;
            }
        }
    }
}



/// draw_board draws the board (using text characters) in a pleasant-looking way, converting the board in a string (board_to_string) and then printing this.
pub fn draw_board(game: &game::Game) {

    let board = game.get_board();

    // Declare board_to_string and add column reference at the top
    let mut board_to_string: String = "\n\t   a  b  c  d  e  f  g  h\n".to_string();

    // For every row add a row reference to the left
    for (row, row_array) in board.iter().enumerate() {
        board_to_string.push('\t');
        board_to_string.push_str(&(row + 1).to_string());
        board_to_string.push(' ');

        // For every column, add the appropriate character depending on the content of the current cell
        for (col, cell) in row_array.iter().enumerate() {

            match *cell {
                // Light and Dark cells are represented by white and black bullets
                game::Cell::Taken { player: game::Player::Light } => board_to_string.push_str(" ○ "),
                game::Cell::Taken { player: game::Player::Dark }  => board_to_string.push_str(" ● "),

                // An empty cell will display a plus or a multiplication sign if the current player can move in that cell
                // or a little central dot otherwise
                game::Cell::Empty => {
                    if game.check_move((row, col)) {
                        if let game::Status::Running { next_player } = game.get_status() {
                            match next_player {
                                game::Player::Light => board_to_string.push_str(" + "),
                                game::Player::Dark  => board_to_string.push_str(" × "),
                            }
                        }
                    } else {
                        board_to_string.push_str(" ∙ ");
                    }
                }
            }
        }

        // Add a row reference to the right
        board_to_string.push(' ');
        board_to_string.push_str(&(row + 1).to_string());
        board_to_string.push('\n');
    }

    // Add column reference at the bottom
    board_to_string.push_str("\t   a  b  c  d  e  f  g  h\n");

    // Print board
    println!("{}", board_to_string);

    // Print current score and game info
    let (score_light, score_dark) = game.get_score();

    match game.get_status() {
        game::Status::Running { next_player } => {
            match next_player {
                game::Player::Light => println!("\t        {:>2} ○ << ● {:<2}\n", score_light, score_dark),
                game::Player::Dark  => println!("\t        {:>2} ○ >> ● {:<2}\n", score_light, score_dark),
            }
        }
        game::Status::Ended => {
            println!("\t        {:>2} ○    ● {:<2}\n", score_light, score_dark);
            match score_light.cmp(&score_dark) {
                Ordering::Greater => println!("Light wins!"),
                Ordering::Less    => println!("Dark wins!"),
                Ordering::Equal   => println!("Draw!"),
            }
        }
    }
}


pub fn print_move(game: &game::Game, (row, col): (usize, usize)) {

    let char_col = (('a' as u8) + (col as u8)) as char;
    if let game::Status::Running { next_player } = game.get_status() {
        match next_player {
            game::Player::Light => println!("○ Light moves: {}{}", char_col, row + 1),
            game::Player::Dark  => println!("● Dark moves: {}{}",  char_col, row + 1),
        }
    }
}
