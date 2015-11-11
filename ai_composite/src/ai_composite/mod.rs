extern crate time;

use rusthello_lib::reversi;

mod opening;
mod midgame;
mod endgame;

const BOARD_AREA: u8 = 64;

const MINIMUM_DEPTH: u8 = 6;
const ENDING_DEPTH: u8 = 13;
const TIME_LIMIT: f64 = 1.0;



pub fn make_move(game: &reversi::Game) -> (usize, usize) {

    if game.get_tempo() + ENDING_DEPTH >= BOARD_AREA {
        return endgame::find_best_move(game, BOARD_AREA - game.get_tempo());
    } else {

        let mut depth: u8 = MINIMUM_DEPTH;

        let start_time = time::precise_time_s();
        let mut current_time;

        let mut best_move: (usize, usize) = match game.get_tempo() {
             4 ... 24 => opening::find_best_move(game, MINIMUM_DEPTH),
            25 ... 44 => midgame::find_best_move(game, MINIMUM_DEPTH),
            45 ... 64 => endgame::find_best_move(game, MINIMUM_DEPTH),
            _         => panic!("get_tempo() returned unexpected result in make_move()!"),
        };

        current_time = time::precise_time_s();

        while ( current_time - start_time < TIME_LIMIT ) && ( depth + 1 + game.get_tempo() <= BOARD_AREA ) {
            depth += 1;
            best_move = match game.get_tempo() {
                 4 ... 24 => opening::find_best_move(game, MINIMUM_DEPTH),
                25 ... 44 => midgame::find_best_move(game, MINIMUM_DEPTH),
                45 ... 64 => endgame::find_best_move(game, MINIMUM_DEPTH),
                _         => panic!("get_tempo() returned unexpected result in make_move()!"),
            };
            current_time = time::precise_time_s();
        }

        return best_move;
    }

}
