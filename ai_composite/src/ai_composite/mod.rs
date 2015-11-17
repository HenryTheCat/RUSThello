extern crate time;

use rusthello_lib::reversi;

mod opening;
mod midgame;
mod endgame;

const BOARD_AREA: u8 = 64;

const MINIMUM_DEPTH: u8 = 6;
const ENDING_DEPTH: u8 = 13;
const TIME_LIMIT: f64 = 1.0;

const LIGHT_STARTING_SCORE: i16 = -10_000;
const DARK_STARTING_SCORE:  i16 =  10_000;

use std::thread;
use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};



pub fn make_move(game: &reversi::Game) -> (usize, usize) {

    // To save computation time, first check whether the move is forced.

    let mut is_forced_move = false;
    let mut forced_move: (usize, usize) = (reversi::BOARD_SIZE, reversi::BOARD_SIZE);
    let mut game_after_move = game.clone();

    'forced_move_loop: for row in 0..reversi::BOARD_SIZE {
        for col in 0..reversi::BOARD_SIZE {
            if game_after_move.make_move((row, col)) {
                if is_forced_move {
                    is_forced_move = false;
                    break 'forced_move_loop;
                } else {
                    is_forced_move = true;
                    forced_move = (row, col);
                }
                game_after_move = game.clone();
            }
        }
    }

    if is_forced_move {

        forced_move

    } else {

        // If the end of the match is close enough, just compute the best move possible.

        if game.get_tempo() + ENDING_DEPTH >= BOARD_AREA {

            find_best_move(game)

        } else {

            // In all the other cases compute the best move up to depth MINIMUM_DEPTH.

            let mut depth: u8 = MINIMUM_DEPTH;

            let start_time = time::precise_time_s();

            let mut best_move: (usize, usize) = match game.get_tempo() {
                 4 ... 15 => opening::find_best_move(game, MINIMUM_DEPTH),
                16 ... 44 => midgame::find_best_move(game, MINIMUM_DEPTH),
                45 ... 64 => endgame::find_best_move(game, MINIMUM_DEPTH),
                _         => panic!("get_tempo() returned unexpected result in make_move()!"),
            };

            // If there's still time left, compute again progressively increasing depth.

            while ( time::precise_time_s() - start_time < TIME_LIMIT ) && ( depth + 1 + game.get_tempo() <= BOARD_AREA ) {
                depth += 1;
                best_move = match game.get_tempo() {
                     4 ... 15 => opening::find_best_move(game, depth),
                    16 ... 44 => midgame::find_best_move(game, depth),
                    45 ... 64 => endgame::find_best_move(game, depth),
                    _         => panic!("get_tempo() returned unexpected result in make_move()!"),
                };
            }

            best_move
        }
    }
}



fn find_best_move(game: &reversi::Game) -> (usize, usize) {

    if let reversi::Status::Running { current_player } = game.get_status() {

        let mut best_move: (usize, usize) = (reversi::BOARD_SIZE, reversi::BOARD_SIZE);
        let mut moves_num: u8 = 0;
        let mut best_score: i16 = match current_player {
            reversi::Player::Light => LIGHT_STARTING_SCORE,
            reversi::Player::Dark  => DARK_STARTING_SCORE,
        };

        let (tx, rx): (Sender<((usize, usize), i16)>, Receiver<((usize, usize), i16)>) = mpsc::channel();

        let mut game_after_move = game.clone();

        for row in 0..reversi::BOARD_SIZE {
            for col in 0..reversi::BOARD_SIZE {
                if game_after_move.make_move((row, col)) {

                    moves_num +=1;
                    let thread_tx = tx.clone();

                    thread::spawn(move || {
                        let current_score = eval(&game_after_move);
                        thread_tx.send(((row, col), current_score)).unwrap();
                    });

                    game_after_move = game.clone();
                }
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

        best_move
    } else {
        panic!{"Game ended, cannot make a move!"};
    }
}



fn eval(game: &reversi::Game) -> i16 {

    match game.get_status() {
        reversi::Status::Running { current_player } => {
            match current_player {
                reversi::Player::Light => {

                    let mut best_score: i16 = LIGHT_STARTING_SCORE;
                    let mut current_score: i16;
                    let mut game_after_move = game.clone();

                    for row in 0..reversi::BOARD_SIZE {
                        for col in 0..reversi::BOARD_SIZE {
                            if game_after_move.make_move((row, col)) {

                                current_score = eval(&game_after_move);
                                if current_score > best_score {
                                    best_score = current_score;
                                }
                                game_after_move = game.clone();

                            }
                        }
                    }

                    best_score

                }
                reversi::Player::Dark => {

                    let mut best_score: i16 =  DARK_STARTING_SCORE;
                    let mut current_score: i16;
                    let mut game_after_move = game.clone();

                    for row in 0..reversi::BOARD_SIZE {
                        for col in 0..reversi::BOARD_SIZE {
                            if game_after_move.make_move((row, col)) {

                                current_score = eval(&game_after_move);
                                if current_score < best_score {
                                    best_score = current_score;
                                }
                                game_after_move = game.clone();
                            }
                        }
                    }

                    best_score

                }
            }
        }
        reversi::Status::Ended => {

            game.get_score_diff()
            
        }
    }
}
