extern crate time;

use std::thread;
use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};

use rusthello_lib::reversi;

const BOARD_AREA: u8 = 64;

const MINIMUM_DEPTH: u8 = 6;
const ENDING_DEPTH: u8 = 13;
const TIME_LIMIT: f64 = 1.0;

const LIGHT_STARTING_SCORE: i16 = -10_000;
const DARK_STARTING_SCORE:  i16 =  10_000;

const RANDOMNESS: i16 = 1;

const BONUS_TURN: i16 = 3;

const MOBILITY: i16 = 1;

enum EvalReturn {
    Ended,
    Running { fixed_cells_board: reversi::Board }
}



pub fn make_move(game: &reversi::Game) -> (usize, usize) {

    if game.get_tempo() + ENDING_DEPTH >= BOARD_AREA {
        return find_best_move(game, BOARD_AREA - game.get_tempo());
    } else {

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

        return best_move;
    }

}



fn find_best_move(game: &reversi::Game, depth: u8) -> (usize, usize) {

    if let reversi::Status::Running { current_player } = game.get_status() {

        if depth > 0 {

            let mut best_move: (usize, usize) = (reversi::BOARD_SIZE, reversi::BOARD_SIZE);
            let mut best_score: i16;

            let mut best_end_move: (usize, usize) = (reversi::BOARD_SIZE, reversi::BOARD_SIZE);
            let mut best_end_score: i16;

            let mut num_moves: u8 = 0;
            let mut end_game: bool = true;

            match current_player {
                reversi::Player::Light => {
                    best_score = LIGHT_STARTING_SCORE;
                    best_end_score = LIGHT_STARTING_SCORE;
                }
                reversi::Player::Dark  => {
                    best_score = DARK_STARTING_SCORE;
                    best_end_score = DARK_STARTING_SCORE;
                }
            }

            let (tx, rx): (Sender<((usize, usize), (i16, EvalReturn))>, Receiver<((usize, usize), (i16, EvalReturn))>) = mpsc::channel();

            let mut game_after_move;

            for row in 0..reversi::BOARD_SIZE {
                for col in 0..reversi::BOARD_SIZE {
                    game_after_move = game.clone();
                    if game_after_move.make_move((row, col)) {
                        num_moves +=1;

                        let thread_tx = tx.clone();

                        thread::spawn(move || {
                            thread_tx.send(( (row, col), eval(&game_after_move, depth - 1) )).unwrap();
                        });
                    }
                }
            }

            for _ in 0..num_moves {
                let (current_move, (current_score, current_eval_return)) = rx.recv().ok().expect("Could not receive answer");

                match current_player {
                    reversi::Player::Light => {
                        if let EvalReturn::Ended = current_eval_return {
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
                    reversi::Player::Dark  => {
                        if let EvalReturn::Ended = current_eval_return {
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

            match current_player {
                reversi::Player::Light  => {
                    if best_end_score > 0 || (best_end_score == 0 && best_score < 0) || end_game {
                        return best_end_move;
                    } else {
                        return best_move;
                    }
                }
                reversi::Player::Dark  => {
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



fn eval(game: &reversi::Game, depth: u8) -> (i16, EvalReturn) {

    match game.get_status() {
        reversi::Status::Ended => (game.get_score_diff(), EvalReturn::Ended),
        reversi::Status::Running { current_player } => {
            if depth == 0 {
                match current_player {
                    reversi::Player::Light => ( BONUS_TURN, EvalReturn::Running { fixed_cells_board: game.get_board() }),
                    reversi::Player::Dark  => (-BONUS_TURN, EvalReturn::Running { fixed_cells_board: game.get_board() }),
                }
            } else {

                match current_player {

                    reversi::Player::Light => {
                        let mut end_game: bool = true;
                        let mut num_moves: i16 = 0;
                        let mut best_score = LIGHT_STARTING_SCORE;
                        let mut best_end_score: i16 = LIGHT_STARTING_SCORE;
                        let mut fixed_cells_boards: Vec<reversi::Board> = Vec::new();
                        let mut game_after_move;

                        for row in 0..reversi::BOARD_SIZE {
                            for col in 0..reversi::BOARD_SIZE {
                                game_after_move = game.clone();
                                if game_after_move.make_move((row, col)) {

                                    let (current_score, current_eval_return) = eval(&game_after_move, depth - 1);

                                    if let EvalReturn::Ended = current_eval_return {
                                        if current_score > best_end_score {
                                            best_end_score = current_score;
                                        }
                                    } else if let EvalReturn::Running { fixed_cells_board: current_fixed_cells_board} = current_eval_return {
                                        num_moves += 1;
                                        end_game = false;
                                        fixed_cells_boards.push(current_fixed_cells_board);
                                        if current_score > best_score {
                                            best_score = current_score;
                                        }
                                    }

                                }
                            }
                        }

                        if end_game || best_end_score > 0 || (best_end_score == 0 && best_score < 0) {
                            (best_end_score, EvalReturn::Ended)
                        } else {
                            let (fixed_cells_board, score_diff) = board_intersec(&mut fixed_cells_boards);
                            (best_score + MOBILITY*num_moves + score_diff, EvalReturn::Running { fixed_cells_board: fixed_cells_board })
                        }
                    }

                    reversi::Player::Dark  => {
                        let mut end_game: bool = true;
                        let mut num_moves: i16 = 0;
                        let mut best_score = DARK_STARTING_SCORE;
                        let mut best_end_score: i16 = DARK_STARTING_SCORE;
                        let mut fixed_cells_boards: Vec<reversi::Board> = Vec::new();
                        let mut game_after_move;

                        for row in 0..reversi::BOARD_SIZE {
                            for col in 0..reversi::BOARD_SIZE {
                                game_after_move = game.clone();
                                if game_after_move.make_move((row, col)) {

                                    let (current_score, current_eval_return) = eval(&game_after_move, depth - 1);

                                    if let EvalReturn::Ended = current_eval_return {
                                        if current_score < best_end_score {
                                            best_end_score = current_score;
                                        }
                                    } else if let EvalReturn::Running { fixed_cells_board: current_fixed_cells_board} = current_eval_return {
                                        num_moves += 1;
                                        end_game = false;
                                        fixed_cells_boards.push(current_fixed_cells_board);
                                        if current_score < best_score {
                                            best_score = current_score;
                                        }
                                    }

                                }
                            }
                        }

                        if end_game || best_end_score < 0 || (best_end_score == 0 && best_score > 0) {
                            (best_end_score, EvalReturn::Ended)
                        } else {
                            let (fixed_cells_board, score_diff) = board_intersec(&mut fixed_cells_boards);
                            (best_score - MOBILITY*num_moves + score_diff, EvalReturn::Running { fixed_cells_board: fixed_cells_board })
                        }
                    }

                }

            }
        }
    }
}


fn board_intersec(boards: &mut Vec<reversi::Board>) -> (reversi::Board, i16) {

    let mut intersection_board: reversi::Board = [[reversi::Cell::Empty; reversi::BOARD_SIZE]; reversi::BOARD_SIZE];

    if let Some(first_board) = boards.pop() {

        let mut score_diff: i16 = 0;
        for (row, row_array) in first_board.iter().enumerate() {
            'cell_loop: for (col, cell) in row_array.iter().enumerate() {
                match *cell {
                    reversi::Cell::Taken { disk: reversi::Player::Light } => {
                        for next_board in boards.iter() {
                            if let reversi::Cell::Taken { disk: reversi::Player::Light } = next_board[row][col] {
                                continue;
                            } else {
                                continue 'cell_loop;
                            }
                        }
                        intersection_board[row][col] = reversi::Cell::Taken { disk: reversi::Player::Light };
                        score_diff += 1;
                    }
                    reversi::Cell::Taken { disk: reversi::Player::Dark } => {
                        for next_board in boards.iter() {
                            if let reversi::Cell::Taken { disk: reversi::Player::Dark } = next_board[row][col] {
                                continue;
                            } else {
                                continue 'cell_loop;
                            }
                        }
                        intersection_board[row][col] = reversi::Cell::Taken { disk: reversi::Player::Dark };
                        score_diff -= 1;
                    }
                    _ => {}
                }
            }
        }

        return (intersection_board, score_diff);

    } else {
        return (intersection_board, 0);
    }

}



/*

fn eval(game: &reversi::Game, depth: u8) -> (i16, bool) {

    match game.get_status() {
        reversi::Status::Ended => (game.get_score_diff(), true),
        reversi::Status::Running { current_player } => {
            if depth == 0 {
                match current_player {
                    reversi::Player::Light => (heavy_eval(game) + BONUS_TURN, false),
                    reversi::Player::Dark  => (heavy_eval(game) - BONUS_TURN, false),
                }
            } else {

                match current_player {

                    reversi::Player::Light => {
                        let mut end_game: bool = true;
                        let mut num_moves: i16 = 0;
                        let mut best_score = LIGHT_STARTING_SCORE;
                        let mut best_end_score: i16 = LIGHT_STARTING_SCORE;
                        let mut game_after_move = game.clone();

                        for row in 0..reversi::BOARD_SIZE {
                            for col in 0..reversi::BOARD_SIZE {
                                if game_after_move.make_move((row, col)) {

                                    let (current_score, current_end_game) = eval(&game_after_move, depth - 1);

                                    if current_end_game && current_score > best_end_score {
                                        best_end_score = current_score;
                                    } else {
                                        num_moves += 1;
                                        end_game = false;
                                        if current_score > best_score {
                                            best_score = current_score;
                                        }
                                    }
                                }
                                game_after_move = game.clone();
                            }
                        }

                        if end_game || best_end_score > 0 || (best_end_score == 0 && best_score < 0) {
                            (best_end_score, true)
                        } else {
                            (best_score + MOBILITY*num_moves, false)
                        }
                    }

                    reversi::Player::Dark  => {
                        let mut end_game: bool = true;
                        let mut num_moves: i16 = 0;
                        let mut best_score = DARK_STARTING_SCORE;
                        let mut best_end_score: i16 = DARK_STARTING_SCORE;
                        let mut game_after_move = game.clone();

                        for row in 0..reversi::BOARD_SIZE {
                            for col in 0..reversi::BOARD_SIZE {
                                if game_after_move.make_move((row, col)) {

                                    let (current_score, current_end_game) = eval(&game_after_move, depth - 1);

                                    if current_end_game && current_score < best_end_score {
                                        best_end_score = current_score;
                                    } else {
                                        end_game = false;
                                        num_moves += 1;
                                        if current_score < best_score {
                                            best_score = current_score;
                                        }
                                    }
                                }
                                game_after_move = game.clone();
                            }
                        }

                        if end_game || best_end_score < 0 || (best_end_score == 0 && best_score > 0) {
                            (best_end_score, true)
                        } else {
                            (best_score - MOBILITY*num_moves, false)
                        }
                    }

                }

            }
        }
    }
}



fn heavy_eval(game: &reversi::Game) -> i16 {
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
    let mut score: i16 = ( game.get_score_diff() * FIXED_BONUS * game.get_tempo() as i16 ) / 64; // (score_light as i16) - (score_dark as i16);

    for &(corner, odd, odd_corner, even, even_corner, counter_odd, counter_even) in SIDES.iter() {

        if let reversi::Cell::Taken { disk } = game.get_cell(corner) {
            match disk {
                reversi::Player::Light => {
                    score += CORNER_BONUS;
                    if let reversi::Cell::Taken { disk: reversi::Player::Light } = game.get_cell(odd) {
                        score += FIXED_BONUS;
                        if let reversi::Cell::Taken { disk: reversi::Player::Light } = game.get_cell(even) {
                            score += FIXED_BONUS;
                        }
                    }
                    if let reversi::Cell::Taken { disk: reversi::Player::Light } = game.get_cell(counter_odd) {
                        score += FIXED_BONUS;
                        if let reversi::Cell::Taken { disk: reversi::Player::Light } = game.get_cell(counter_even) {
                            score += FIXED_BONUS;
                        }
                    }
                }
                reversi::Player::Dark => {
                    score -= CORNER_BONUS;
                    if let reversi::Cell::Taken { disk: reversi::Player::Dark } = game.get_cell(odd) {
                        score -= FIXED_BONUS;
                        if let reversi::Cell::Taken { disk: reversi::Player::Dark } = game.get_cell(even) {
                            score -= FIXED_BONUS;
                        }
                    }
                    if let reversi::Cell::Taken { disk: reversi::Player::Dark } = game.get_cell(counter_odd) {
                        score -= FIXED_BONUS;
                        if let reversi::Cell::Taken { disk: reversi::Player::Dark } = game.get_cell(counter_even) {
                            score -= FIXED_BONUS;
                        }
                    }
                }
            }

        } else {

            if let reversi::Cell::Taken { disk } = game.get_cell(odd) {
                match disk {
                    reversi::Player::Light => score -= ODD_MALUS,
                    reversi::Player::Dark  => score += ODD_MALUS,
                }
            } else if let reversi::Cell::Taken { disk } = game.get_cell(even) {
                match disk {
                    reversi::Player::Light => score += EVEN_BONUS,
                    reversi::Player::Dark  => score -= EVEN_BONUS,
                }
            }

            if let reversi::Cell::Taken { disk } = game.get_cell(counter_odd) {
                match disk {
                    reversi::Player::Light => score -= ODD_MALUS,
                    reversi::Player::Dark  => score += ODD_MALUS,
                }
            } else if let reversi::Cell::Taken { disk } = game.get_cell(counter_even) {
                match disk {
                    reversi::Player::Light => score += EVEN_BONUS,
                    reversi::Player::Dark  => score -= EVEN_BONUS,
                }
            }

            if let reversi::Cell::Taken { disk } = game.get_cell(odd_corner) {
                match disk {
                    reversi::Player::Light => score -= ODD_CORNER_MALUS,
                    reversi::Player::Dark  => score += ODD_CORNER_MALUS,
                }

            } else if let reversi::Cell::Taken { disk } = game.get_cell(even_corner) {
                match disk {
                    reversi::Player::Light => score += EVEN_CORNER_BONUS,
                    reversi::Player::Dark  => score -= EVEN_CORNER_BONUS,
                }
            }
        }
    }

    score
}

*/
