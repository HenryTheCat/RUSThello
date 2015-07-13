extern crate time;

use std::thread;
use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};

use game;

const BOARD_AREA: u8 = 64;

const STARTING_DEPTH: u8 = 5;
const TIME_LIMIT: f64 = 1.0;

const LIGHT_STARTING_SCORE: i16 = -10_000;
const DARK_STARTING_SCORE:  i16 =  10_000;

const RANDOMNESS: i16 = 1;

const BONUS_TURN: i16 = 3;

const MOBILITY: i16 = 1;



pub fn make_move(game: &game::Game) -> (usize, usize) {

    let mut start_time = time::precise_time_s();
    let mut end_time;
    let mut depth = STARTING_DEPTH;

    let mut best_move: (usize, usize)  = find_best_move(game, depth);

    end_time = time::precise_time_s();

    while ( end_time - start_time < TIME_LIMIT ) && ( depth + 1 + game.get_turn() <= BOARD_AREA ) {
        depth += 1;
        start_time = time::precise_time_s();
        best_move = find_best_move(game, depth);
        end_time = time::precise_time_s();
    }

    best_move
}



fn find_best_move(game: &game::Game, depth: u8) -> (usize, usize) {

    if let game::Status::Running { next_player } = game.get_status() {

        if depth > 0 {

            let mut best_move: (usize, usize) = (game::BOARD_SIZE, game::BOARD_SIZE);
            let mut best_score: i16;

            let mut best_end_move: (usize, usize) = (game::BOARD_SIZE, game::BOARD_SIZE);
            let mut best_end_score: i16;

            let mut num_moves: u8 = 0;
            let mut end_game: bool = true;

            match next_player {
                game::Player::Light => {
                    best_score = LIGHT_STARTING_SCORE;
                    best_end_score = LIGHT_STARTING_SCORE;
                }
                game::Player::Dark  => {
                    best_score = DARK_STARTING_SCORE;
                    best_end_score = DARK_STARTING_SCORE;
                }
            }

            let (tx, rx): (Sender<((usize, usize), (i16, bool))>, Receiver<((usize, usize), (i16, bool))>) = mpsc::channel();

            for row in 0..game::BOARD_SIZE {
                for col in 0..game::BOARD_SIZE {

                    if game.check_move((row, col)) {
                        num_moves +=1;

                        let thread_tx = tx.clone();
                        let mut game = game.clone();

                        thread::spawn(move || {
                            thread_tx.send(( (row, col), eval(game.make_move((row, col)), depth - 1) )).unwrap();
                        });
                    }
                }
            }

            for _ in 0..num_moves {
                let (current_move, (current_score, current_end_game)) = rx.recv().ok().expect("Could not receive answer");

                match next_player {
                    game::Player::Light => {
                        if current_end_game {
                            if current_score > best_end_score {
                                best_end_score = current_score;
                                best_end_move = current_move;
                            }
                        } else {
                            if current_score + RANDOMNESS > best_score {
                                best_score = current_score;
                                best_move = current_move;
                                end_game = false;
                            }
                        }
                    }
                    game::Player::Dark  => {
                        if current_end_game {
                            if current_score < best_end_score {
                                best_end_score = current_score;
                                best_end_move = current_move;
                            }
                        } else {
                            if current_score - RANDOMNESS < best_score {
                                best_score = current_score;
                                best_move = current_move;
                                end_game = false;
                            }
                        }
                    }
                }
            }

            match next_player {
                game::Player::Light  => {
                    if best_end_score > 0 || (best_end_score == 0 && best_score < 0) || end_game {
                        return best_end_move;
                    } else {
                        return best_move;
                    }
                }
                game::Player::Dark  => {
                    if best_end_score < 0 || (best_end_score == 0 && best_score > 0) || end_game {
                        return best_end_move;
                    } else {
                        return best_move;
                    }
                }
            }

        } else {
            panic!("Depth cannot be zero");
        }

    } else {
        panic!{"Game ended, cannot make a move!"};
    }
}



fn eval(game: &game::Game, depth: u8) -> (i16, bool) {

    match game.get_status() {
        game::Status::Ended => return (game.get_score_diff(), true),
        game::Status::Running { next_player } => {
            if depth == 0 {
                match next_player {
                    game::Player::Light => return (heavy_eval(game) + BONUS_TURN, false),
                    game::Player::Dark  => return (heavy_eval(game) - BONUS_TURN, false),
                }
            } else {
                let mut end_game: bool = true;
                let mut num_moves: i16 = 0;
                let mut best_end_score: i16;
                let mut best_score: i16;

                match next_player {

                    game::Player::Light => {
                        best_score = LIGHT_STARTING_SCORE;
                        best_end_score = LIGHT_STARTING_SCORE;

                        for row in 0..game::BOARD_SIZE {
                            for col in 0..game::BOARD_SIZE {
                                if game.check_move((row, col)) {

                                    let (current_score, current_end_game) = eval(game.clone().make_move((row, col)), depth - 1);

                                    if current_end_game {
                                        if current_score > best_end_score {
                                            best_end_score = current_score;
                                        }
                                    } else {
                                        num_moves += 1;
                                        if current_score > best_score {
                                            best_score = current_score;
                                            end_game = false;
                                        }
                                    }

                                }
                            }
                        }

                        if best_end_score > 0 || (best_end_score == 0 && best_score < 0) || end_game {
                            return (best_end_score, true);
                        } else {
                            return (best_score + MOBILITY*num_moves, false);
                        }
                    }

                    game::Player::Dark  => {
                        best_score = DARK_STARTING_SCORE;
                        best_end_score = DARK_STARTING_SCORE;

                        for row in 0..game::BOARD_SIZE {
                            for col in 0..game::BOARD_SIZE {
                                if game.check_move((row, col)) {

                                    let (current_score, current_end_game) = eval(game.clone().make_move((row, col)), depth - 1);

                                    if current_end_game {
                                        if current_score < best_end_score {
                                            best_end_score = current_score;
                                        }
                                    } else {
                                        num_moves += 1;
                                        if current_score < best_score {
                                            best_score = current_score;
                                            end_game = false;
                                        }
                                    }

                                }
                            }
                        }

                        if best_end_score < 0 || (best_end_score == 0 && best_score > 0) || end_game {
                            return (best_end_score, true);
                        } else {
                            return (best_score - MOBILITY*num_moves, false);
                        }
                    }

                }

            }
        }
    }
}



fn heavy_eval(game: &game::Game) -> i16 {
    const CORNER_BONUS: i16 = 15;
    const ODD_MALUS: i16 = 3;
    const EVEN_BONUS: i16 = 3;
    const ODD_CORNER_MALUS: i16 = 10;
    const EVEN_CORNER_BONUS: i16 = 5;
    const FIXED_BONUS: i16 = 3;

    const SIDES: [( (usize, usize), (usize, usize), (usize, usize), (usize, usize), (usize, usize), (usize, usize), (usize, usize) ); 4] = [
        ( (0,0), (0,1), (1,1), (0,2), (2,2), (1,0), (2,0) ), // NW corner
        ( (0,7), (1,7), (1,6), (2,7), (2,5), (0,6), (0,5) ), // NE corner
        ( (7,0), (6,0), (6,1), (5,0), (5,2), (7,1), (7,2) ), // SW corner
        ( (7,7), (6,7), (6,6), (5,7), (5,5), (7,6), (7,6) ), // SE corner
        ];


    //let (score_light, score_dark) = game.get_score();
    let mut score: i16 = 0; // (score_light as i16) - (score_dark as i16);

    for special_cells in SIDES.iter() {

        let (corner, odd, odd_corner, even, even_corner, counter_odd, counter_even) = *special_cells;

        if let game::Cell::Taken { player } = game.get_cell(corner) {
            match player {
                game::Player::Light => {
                    score += CORNER_BONUS;
                    if let game::Cell::Taken { player: game::Player::Light } = game.get_cell(odd) {
                        score += FIXED_BONUS;
                        if let game::Cell::Taken { player: game::Player::Light } = game.get_cell(even) {
                            score += FIXED_BONUS;
                        }
                    }
                    if let game::Cell::Taken { player: game::Player::Light } = game.get_cell(counter_odd) {
                        score += FIXED_BONUS;
                        if let game::Cell::Taken { player: game::Player::Light } = game.get_cell(counter_even) {
                            score += FIXED_BONUS;
                        }
                    }
                }
                game::Player::Dark => {
                    score -= CORNER_BONUS;
                    if let game::Cell::Taken { player: game::Player::Dark } = game.get_cell(odd) {
                        score -= FIXED_BONUS;
                        if let game::Cell::Taken { player: game::Player::Dark } = game.get_cell(even) {
                            score -= FIXED_BONUS;
                        }
                    }
                    if let game::Cell::Taken { player: game::Player::Dark } = game.get_cell(counter_odd) {
                        score -= FIXED_BONUS;
                        if let game::Cell::Taken { player: game::Player::Dark } = game.get_cell(counter_even) {
                            score -= FIXED_BONUS;
                        }
                    }
                }
            }

        } else {

            if let game::Cell::Taken { player } = game.get_cell(odd) {
                match player {
                    game::Player::Light => score -= ODD_MALUS,
                    game::Player::Dark  => score += ODD_MALUS,
                }
            } else if let game::Cell::Taken { player } = game.get_cell(even) {
                match player {
                    game::Player::Light => score += EVEN_BONUS,
                    game::Player::Dark  => score -= EVEN_BONUS,
                }
            }

            if let game::Cell::Taken { player } = game.get_cell(counter_odd) {
                match player {
                    game::Player::Light => score -= ODD_MALUS,
                    game::Player::Dark  => score += ODD_MALUS,
                }
            } else if let game::Cell::Taken { player } = game.get_cell(counter_even) {
                match player {
                    game::Player::Light => score += EVEN_BONUS,
                    game::Player::Dark  => score -= EVEN_BONUS,
                }
            }

            if let game::Cell::Taken { player } = game.get_cell(odd_corner) {
                match player {
                    game::Player::Light => score -= ODD_CORNER_MALUS,
                    game::Player::Dark  => score += ODD_CORNER_MALUS,
                }

            } else if let game::Cell::Taken { player } = game.get_cell(even_corner) {
                match player {
                    game::Player::Light => score += EVEN_CORNER_BONUS,
                    game::Player::Dark  => score -= EVEN_CORNER_BONUS,
                }
            }
        }
    }

    score
}
