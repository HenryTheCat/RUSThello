
extern crate time;

//use std::sync::{Arc, Mutex};
use std::thread;
use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};

use rusthello_lib::game;

const BOARD_AREA: u8 = 64; // Mysterious casting error //( game::BOARD_SIZE*game::BOARD_SIZE);

const MINIMUM_DEPTH: u8 = 5;
const TIME_LIMIT: f64 = 0.5;

const LIGHT_STARTING_SCORE: i16 = -10_000;
const DARK_STARTING_SCORE:  i16 =  10_000;

const BONUS_TURN: i16 = 3;



pub fn make_move(game: &game::Game) -> (usize, usize) {

    let mut depth: u8 = MINIMUM_DEPTH;

    let start_time = time::precise_time_s();
    let mut current_time;

    let mut best_move: (usize, usize)  = find_best_move(game, MINIMUM_DEPTH);

    current_time = time::precise_time_s();

    while ( current_time - start_time < TIME_LIMIT ) && ( depth + 1 + game.get_turn() <= BOARD_AREA ) {
        depth += 1;
        best_move = find_best_move(game, depth);
        current_time = time::precise_time_s();
    }

    best_move

}



fn find_best_move(game: &game::Game, depth: u8) -> (usize, usize) {

    if let game::Status::Running { next_player } = game.get_status() {

        let mut best_move: (usize, usize) = (game::BOARD_SIZE, game::BOARD_SIZE);
        let mut best_score: i16;
        let starting_score: i16;
        let mut moves_num: u8 = 0;

        match next_player {
            game::Player::Light => starting_score = LIGHT_STARTING_SCORE,
            game::Player::Dark  => starting_score = DARK_STARTING_SCORE,
        }
        best_score = starting_score;

        let (tx, rx): (Sender<((usize, usize), i16)>, Receiver<((usize, usize), i16)>) = mpsc::channel();

        for row in 0..game::BOARD_SIZE {
            for col in 0..game::BOARD_SIZE {

                if game.check_move((row, col)) {
                    moves_num +=1;

                    let thread_tx = tx.clone();
                    let mut game = game.clone();

                    thread::spawn(move || {

                        let current_score = eval(game.make_move((row, col)), depth - 1);

                        thread_tx.send(((row, col), current_score)).unwrap();
                    });
                }
            }
        }

        for _ in 0..moves_num {
            let (current_move, current_score) = rx.recv().ok().expect("Could not receive answer");

            match next_player {
                game::Player::Light => {
                    if current_score > best_score {
                        best_move = current_move;
                        best_score = current_score;
                    }
                }
                game::Player::Dark  => {
                    if current_score < best_score {
                        best_move = current_move;
                        best_score = current_score;
                    }
                }
            }
        }

        return best_move;
    } else {
        panic!{"Game ended, cannot make a move!"};
    }
}



fn eval(game: &game::Game, depth: u8) -> i16 {

    match game.get_status() {
        game::Status::Running { next_player } => {
            if depth == 0 {
                match next_player {
                    game::Player::Light => return game.get_score_diff() + BONUS_TURN,
                    game::Player::Dark  => return game.get_score_diff() - BONUS_TURN,
                }
            } else {
                match next_player {
                    game::Player::Light => {
                        let mut score: i16 = LIGHT_STARTING_SCORE;
                        let mut current_score: i16;
                        for row in 0..game::BOARD_SIZE {
                            for col in 0..game::BOARD_SIZE {
                                if game.check_move((row, col)) {
                                    current_score = eval(game.clone().make_move((row, col)), depth - 1);
                                    if current_score > score {
                                        score = current_score;
                                    }
                                }
                            }
                        }
                        return score;
                    }
                    game::Player::Dark => {
                        let mut score: i16 =  DARK_STARTING_SCORE;
                        let mut current_score: i16;
                        for row in 0..game::BOARD_SIZE {
                            for col in 0..game::BOARD_SIZE {
                                if game.check_move((row, col)) {
                                    current_score = eval(game.clone().make_move((row, col)), depth - 1);
                                    if current_score < score {
                                        score = current_score;
                                    }
                                }
                            }
                        }
                        return score;
                    }
                }
            }
        }
        game::Status::Ended => {
            return game.get_score_diff()*64;
        }
    }
}
