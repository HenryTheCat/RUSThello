
extern crate time;

use game;

const BOARD_AREA: u8 = 64;

const MINIMUM_DEPTH: u8 = 5;
const TIME_LIMIT: f64 = 0.5;

const LIGHT_STARTING_SCORE: i16 = -10_000;
const DARK_STARTING_SCORE:  i16 =  10_000;



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

    let mut best_move: (usize, usize) = (game::BOARD_SIZE, game::BOARD_SIZE);
    let mut best_score: i16;

    if let game::Status::Running { next_player } = game.get_status() {
        match next_player {
            game::Player::Light => {
                best_score = LIGHT_STARTING_SCORE;
                for row in 0..game::BOARD_SIZE {
                    for col in 0..game::BOARD_SIZE {
                        if game.check_move((row, col)) {
                            let current_score = eval(game.clone().make_move((row, col)), depth - 1);
                            if current_score > best_score {
                                best_move = (row, col);
                                best_score = current_score;
                            }
                        }
                    }
                }
            }
            game::Player::Dark => {
                best_score = DARK_STARTING_SCORE;
                for row in 0..game::BOARD_SIZE {
                    for col in 0..game::BOARD_SIZE {
                        if game.check_move((row, col)) {
                            let current_score = eval(game.clone().make_move((row, col)), depth - 1);
                            if current_score < best_score {
                                best_move = (row, col);
                                best_score = current_score;
                            }
                        }
                    }
                }
            }
        }
    } else {
        panic!{"Game ended, cannot make a move!"};
    }
    
    best_move
}



fn eval(game: &game::Game, depth: u8) -> i16 {

    let mut score: i16;

    match game.get_status() {
        game::Status::Running { next_player } => {
            if depth == 0 {
                const BONUS_TURN: i16 = 1;
                let (score_light, score_dark) = game.get_score();
                score = (score_light as i16) - (score_dark as i16);
                match next_player {
                    game::Player::Light => score += BONUS_TURN,
                    game::Player::Dark  => score -= BONUS_TURN,
                }
            } else {
                match next_player {
                    game::Player::Light => {
                        score =  LIGHT_STARTING_SCORE;
                        for row in 0..game::BOARD_SIZE {
                            for col in 0..game::BOARD_SIZE {
                                if game.check_move((row, col)) {
                                    let current_score = eval(game.clone().make_move((row, col)), depth - 1);
                                    if current_score > score {
                                        score = current_score;
                                    }
                                }
                            }
                        }
                    }
                    game::Player::Dark => {
                        score =  DARK_STARTING_SCORE;
                        for row in 0..game::BOARD_SIZE {
                            for col in 0..game::BOARD_SIZE {
                                if game.check_move((row, col)) {
                                    let current_score = eval(game.clone().make_move((row, col)), depth - 1);
                                    if current_score < score {
                                        score = current_score;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        game::Status::Ended => {
            let (score_light, score_dark) = game.get_score();
            score = (score_light as i16) - (score_dark as i16);
            score *= 64;
        }
    }

    score
}
