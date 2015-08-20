extern crate time;

use rusthello_lib::reversi;

//use std::sync::{Arc, Mutex};
use std::thread;
use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};

const BOARD_AREA: u8 = 64; // Mysterious casting error //( game::BOARD_SIZE*game::BOARD_SIZE);

const MINIMUM_DEPTH: u8 = 5;
const TIME_LIMIT: f64 = 1.0;

const LIGHT_STARTING_SCORE: i16 = -10_000;
const DARK_STARTING_SCORE:  i16 =  10_000;

const BONUS_TURN: i16 = 3;



pub fn make_move(game: &reversi::Game) -> (usize, usize) {

    let mut depth: u8 = MINIMUM_DEPTH;

    let start_time = time::precise_time_s();
    let mut current_time;

    let mut best_move: (usize, usize)  = find_best_move(game, MINIMUM_DEPTH);

    current_time = time::precise_time_s();

    while ( current_time - start_time < TIME_LIMIT ) && ( depth + 1 + game.get_tempo() <= BOARD_AREA ) {
        depth += 1;
        best_move = find_best_move(game, depth);
        current_time = time::precise_time_s();
    }

    best_move

}



fn find_best_move(game: &reversi::Game, depth: u8) -> (usize, usize) {

    if let reversi::Status::Running { current_player } = game.get_status() {

        let mut best_move: (usize, usize) = (reversi::BOARD_SIZE, reversi::BOARD_SIZE);
        let mut best_score: i16;
        let starting_score: i16;
        let mut moves_num: u8 = 0;

        match current_player {
            reversi::Player::Light => starting_score = LIGHT_STARTING_SCORE,
            reversi::Player::Dark  => starting_score = DARK_STARTING_SCORE,
        }
        best_score = starting_score;

        let (tx, rx): (Sender<((usize, usize), i16)>, Receiver<((usize, usize), i16)>) = mpsc::channel();

        let mut game_after_move = game.clone();

        for row in 0..reversi::BOARD_SIZE {
            for col in 0..reversi::BOARD_SIZE {

                if game_after_move.make_move((row, col)) {
                    moves_num +=1;

                    let thread_tx = tx.clone();
                    let game_after_move = game_after_move.clone();

                    thread::spawn(move || {

                        let current_score = eval(&game_after_move, depth - 1);

                        thread_tx.send(((row, col), current_score)).unwrap();
                    });
                }
                game_after_move = game.clone();
            }
        }

        for _ in 0..moves_num {
            let (current_move, current_score) = rx.recv().ok().expect("Could not receive answer");

            match current_player {
                reversi::Player::Light => {
                    if current_score > best_score {
                        best_move = current_move;
                        best_score = current_score;
                    }
                }
                reversi::Player::Dark  => {
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



fn eval(game: &reversi::Game, depth: u8) -> i16 {

    match game.get_status() {
        reversi::Status::Running { current_player } => {
            if depth == 0 {
                match current_player {
                    reversi::Player::Light => return game.get_score_diff() + BONUS_TURN,
                    reversi::Player::Dark  => return game.get_score_diff() - BONUS_TURN,
                }
            } else {
                match current_player {
                    reversi::Player::Light => {
                        let mut score: i16 = LIGHT_STARTING_SCORE;
                        let mut current_score: i16;
                        let mut game_after_move = game.clone();
                        for row in 0..reversi::BOARD_SIZE {
                            for col in 0..reversi::BOARD_SIZE {
                                if game_after_move.make_move((row, col)) {
                                    current_score = eval(&game_after_move, depth - 1);
                                    if current_score > score {
                                        score = current_score;
                                    }
                                    game_after_move = game.clone();
                                }
                            }
                        }
                        return score;
                    }
                    reversi::Player::Dark => {
                        let mut score: i16 =  DARK_STARTING_SCORE;
                        let mut current_score: i16;
                        let mut game_after_move = game.clone();
                        for row in 0..reversi::BOARD_SIZE {
                            for col in 0..reversi::BOARD_SIZE {
                                if game_after_move.make_move((row, col)) {
                                    current_score = eval(&game_after_move, depth - 1);
                                    if current_score < score {
                                        score = current_score;
                                    }
                                    game_after_move = game.clone();
                                }
                            }
                        }
                        return score;
                    }
                }
            }
        }
        reversi::Status::Ended => {
            return game.get_score_diff()*64;
        }
    }
}
