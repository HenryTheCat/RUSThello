extern crate time;

use std::thread;
use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};

use game;

const BOARD_AREA: u8 = 64;

const STARTING_DEPTH: u8 = 5;
const TIME_LIMIT: f64 = 0.5;

const LIGHT_STARTING_SCORE: i16 = -10_000;
const DARK_STARTING_SCORE:  i16 =  10_000;

const RANDOMNESS: i16 = 1;

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
            let mut moves_num: u8 = 0;

            match next_player {
                game::Player::Light => best_score = LIGHT_STARTING_SCORE,
                game::Player::Dark  => best_score = DARK_STARTING_SCORE,
            }

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
                        if current_score + RANDOMNESS > best_score {
                            best_move = current_move;
                            best_score = current_score;
                        }
                    }
                    game::Player::Dark  => {
                        if current_score - RANDOMNESS < best_score {
                            best_move = current_move;
                            best_score = current_score;
                        }
                    }
                }
            }

            return best_move;

        } else {
            panic!("Depth cannot be zero");
        }

    } else {
        panic!{"Game ended, cannot make a move!"};
    }
}


fn eval(game: &game::Game, depth: u8) -> i16 {

    if depth == 0 {
        return basic_eval(game);
    } else {
        match game.get_status() {
            game::Status::Ended => return basic_eval(game),
            game::Status::Running { next_player } => {

                let mut best_score: i16;
                match next_player {
                    game::Player::Light => best_score = LIGHT_STARTING_SCORE,
                    game::Player::Dark  => best_score = DARK_STARTING_SCORE,
                }

                for row in 0..game::BOARD_SIZE {
                    for col in 0..game::BOARD_SIZE {
                        if game.check_move((row, col)) {

                            let current_score = eval(game.clone().make_move((row, col)), depth - 1);

                            match next_player {
                                game::Player::Light => {
                                    if current_score > best_score {
                                        best_score = current_score;
                                    }
                                }
                                game::Player::Dark  => {
                                    if current_score < best_score {
                                        best_score = current_score;
                                    }
                                }
                            }

                        }
                    }
                }

                return best_score;

            }
        }
    }
}



fn basic_eval (game: &game::Game) -> i16 {

    const BONUS_TURN: i16 = 1;

    match game.get_status() {
        game::Status::Running {next_player} => {
            let score: i16 = heavy_eval(game);
            match next_player {
                game::Player::Light => return score + BONUS_TURN,
                game::Player::Dark  => return score - BONUS_TURN,
            }
        }
        game::Status::Ended => {
            let (score_light, score_dark) = game.get_score();
            let score: i16 = (score_light as i16) - (score_dark as i16);
            return score * 64;
        }
    }
}



fn heavy_eval(game: &game::Game) -> i16 {

    const CORNER_BONUS: i16 = 12;
    const ODD_MALUS: i16 = 5;
    const EVEN_BONUS: i16 = 3;
    const ODD_CORNER_MALUS: i16 = 8;
    const EVEN_CORNER_BONUS: i16 = 5;
    const FIXED_BONUS: i16 = 2;

    let (score_light, score_dark) = game.get_score();
    let mut score: i16 = (score_light as i16) - (score_dark as i16);


    const SIDES: [( (usize, usize), (usize, usize), (usize, usize), (usize, usize), (usize, usize), (usize, usize), (usize, usize) ); 4] = [
        ( (0,0), (0,1), (1,1), (0,2), (2,2), (1,0), (2,0) ), // NW corner
        ( (0,7), (1,7), (1,6), (2,7), (2,5), (0,6), (0,5) ), // NE corner
        ( (7,0), (6,0), (6,1), (5,0), (5,2), (7,1), (7,2) ), // SW corner
        ( (7,7), (6,7), (6,6), (5,7), (5,5), (7,6), (7,6) ), // SE corner
        ];

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
