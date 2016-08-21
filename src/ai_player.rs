//! Provides `game::IsPlayer<::OtherAction>` types.

use reversi;
use reversi::{board, turn, game};
use reversi::board::{Coord};
use ::{Result, Action};
use std::cmp::Ordering;
use std::thread;
use rand;
use rand::distributions::{IndependentSample, Range};

const RANDOMNESS: f64 = 0.05f64;
const WEAK: f64 = 100f64;
const MEDIUM: f64 = 1000f64;
const STRONG: f64 = 10000f64;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Score {
    Running(f64),
    Ended(i16),
}

impl PartialOrd<Score> for Score {
    fn partial_cmp(&self, other: &Score) -> Option<Ordering> {
        if *self == *other {
            Some(Ordering::Equal)
        } else if match *self {
            Score::Running(val1) => {
                match *other {
                    Score::Running(val2) => val1 > val2,
                    Score::Ended(scr2) => scr2 < 0i16 || ( scr2 == 0i16 && val1 > 0f64 ),
                }
            }
            Score::Ended(scr1) => {
                match *other {
                    Score::Running(val2) => scr1 > 0i16 || ( scr1 == 0i16 && val2 < 0f64 ),
                    Score::Ended(scr2) => scr1 > scr2,
                }
            }
        } {
            Some(Ordering::Greater)
        } else {
            Some(Ordering::Less)
        }
    }
}

pub enum AiPlayer {
    Weak,
    Medium,
    Strong,
}

impl game::IsPlayer<::OtherAction> for AiPlayer {
    /// Calls `find_best_move` with suitable parameters
    fn make_move(&self, turn: &turn::Turn) -> Result<Action> {
        Ok(game::PlayerAction::Move(try!( match *self {
            AiPlayer::Weak   => AiPlayer::find_best_move(turn, WEAK),
            AiPlayer::Medium => AiPlayer::find_best_move(turn, MEDIUM),
            AiPlayer::Strong => AiPlayer::find_best_move(turn, STRONG),
        })))
    }
}

impl AiPlayer {
    /// Find best moves among the legal ones.
    /// Each possibility is evaluated by a method depending on the value of `self` and confronted with the others.
    pub fn find_best_move(turn: &turn::Turn, depth: f64) -> Result<board::Coord> {

        let side = try!(turn.get_state().ok_or(reversi::ReversiError::EndedGame(*turn)));
        let mut threadjoins = Vec::new();

        for index in 0..board::NUM_CELLS {
            let coord = try!(board::Coord::from_index(index));
            if let Ok(turn_after_move) = turn.check_and_move(coord) {
                threadjoins.push(thread::spawn(move || Ok((coord, try!(AiPlayer::ai_eval(&turn_after_move, depth)))) ));
            }
        }

        let mut some_best_move: Option<(board::Coord, Score)> = None;
        for threadjoin in threadjoins {
            let (new_coord, new_score) = try!(threadjoin.join().expect("Could not receive answer"));
            if let Some((_, best_score)) = some_best_move {
                match side {
                    reversi::Side::Dark  if new_score < best_score => some_best_move = Some((new_coord, new_score)),
                    reversi::Side::Light if new_score > best_score => some_best_move = Some((new_coord, new_score)),
                    // If the new score is not better than the previous one, do nothing.
                    _ => {},
                }
            } else {
                some_best_move = Some((new_coord, new_score));
            }
        }

        some_best_move.ok_or(reversi::ReversiError::EndedGame(*turn)).map(|best_move| best_move.0)
    }

    fn ai_eval(turn: &turn::Turn, depth: f64) -> Result<Score> {
        match *turn.get_state() {
            None => Ok(Score::Ended(turn.get_score_diff())),
            Some(side) => {
                if depth < 1f64 {
                    let val = try!(AiPlayer::heavy_eval(&turn));
                    Ok(Score::Running(val))
                } else {
                    let mut moves: [Option<Coord>; board::NUM_CELLS] = [None; board::NUM_CELLS];
                    let mut num_moves = 0;
                    for index in 0..board::NUM_CELLS {
                        let coord = try!(board::Coord::from_index(index));
                        if turn.check_move(coord).is_ok() {
                            moves[num_moves] = Some(coord);
                            num_moves += 1;
                        }
                    }

                    let mut some_best_score: Option<Score> = None;
                    for &some_coord in &moves[0..num_moves] {
                        if let Some(coord) = some_coord {
                            let turn_after_move = try!(turn.check_and_move(coord));
                            let new_score = try!(AiPlayer::ai_eval(&turn_after_move, depth / num_moves as f64));
                            if let Some(best_score) = some_best_score {
                                match side {
                                    reversi::Side::Dark  if new_score < best_score => some_best_score = Some(new_score),
                                    reversi::Side::Light if new_score > best_score => some_best_score = Some(new_score),
                                    // If the new score is not better than the previous one, do nothing.
                                    _ => {},
                                }
                            } else {
                                some_best_score = Some(new_score);
                            }
                        }
                    }

                    let best_score = try!(some_best_score.ok_or(reversi::ReversiError::EndedGame(*turn)));

                    let between = Range::new(-RANDOMNESS, RANDOMNESS);
                    let mut rng = rand::thread_rng();
                    let best_score = match best_score {
                        Score::Running(val) => Score::Running(val * (1.0 + between.ind_sample(&mut rng))),
                        _ => best_score,
                    };

                    Ok(best_score)
                }
            }
        }
    }

    fn heavy_eval(turn: &turn::Turn) -> Result<f64> {
        // weights
        const CORNER_BONUS:         u16 = 45;
        const ODD_CORNER_MALUS:     u16 = 25;
        const EVEN_CORNER_BONUS:    u16 = 10;
        const ODD_MALUS:            u16 = 6; // x2
        const EVEN_BONUS:           u16 = 4; // x2
        // ------------------------ Sum = 100

        const SIDES: [( (usize, usize), (usize, usize), (usize, usize), (usize, usize), (usize, usize), (usize, usize), (usize, usize) ); 4] = [
            ( (0,0), (0,1), (1,1), (0,2), (2,2), (1,0), (2,0) ), // NW corner
            ( (0,7), (1,7), (1,6), (2,7), (2,5), (0,6), (0,5) ), // NE corner
            ( (7,0), (6,0), (6,1), (5,0), (5,2), (7,1), (7,2) ), // SW corner
            ( (7,7), (6,7), (6,6), (5,7), (5,5), (7,6), (7,5) ), // SE corner
            ];

        let mut score_light: u16 = 1;
        let mut score_dark:  u16 = 1;

        for &(corner, odd, odd_corner, even, even_corner, counter_odd, counter_even) in &SIDES {

            if let &Some(disk) = try!(turn.get_cell(Coord::new(corner.0, corner.1))) {
                match disk.get_side() {
                    reversi::Side::Light => {
                        score_light += CORNER_BONUS;
                    }
                    reversi::Side::Dark => {
                        score_dark  += CORNER_BONUS;
                    }
                }
            } else {

                if let &Some(disk) = try!(turn.get_cell(Coord::new(odd.0, odd.1))) {
                    match disk.get_side() {
                        reversi::Side::Light => score_dark  += ODD_MALUS,
                        reversi::Side::Dark  => score_light += ODD_MALUS,
                    }
                } else if let &Some(disk) = try!(turn.get_cell(Coord::new(even.0, even.1))) {
                    match disk.get_side() {
                        reversi::Side::Light => score_light += EVEN_BONUS,
                        reversi::Side::Dark  => score_dark  += EVEN_BONUS,
                    }
                }

                if let &Some(disk) = try!(turn.get_cell(Coord::new(counter_odd.0, counter_odd.1))) {
                    match disk.get_side() {
                        reversi::Side::Light => score_dark  += ODD_MALUS,
                        reversi::Side::Dark  => score_light += ODD_MALUS,
                    }
                } else if let &Some(disk) = try!(turn.get_cell(Coord::new(counter_even.0, counter_even.1))) {
                    match disk.get_side() {
                        reversi::Side::Light => score_light += EVEN_BONUS,
                        reversi::Side::Dark  => score_dark  += EVEN_BONUS,
                    }
                }

                if let &Some(disk) = try!(turn.get_cell(Coord::new(odd_corner.0, odd_corner.1))) {
                    match disk.get_side() {
                        reversi::Side::Light => score_dark  += ODD_CORNER_MALUS,
                        reversi::Side::Dark  => score_light += ODD_CORNER_MALUS,
                    }

                } else if let &Some(disk) = try!(turn.get_cell(Coord::new(even_corner.0, even_corner.1))) {
                    match disk.get_side() {
                        reversi::Side::Light => score_light += EVEN_CORNER_BONUS,
                        reversi::Side::Dark  => score_dark  += EVEN_CORNER_BONUS,
                    }
                }
            }
        }
        Ok( (score_light as f64 - score_dark as f64 ) / ( score_dark + score_light) as f64 )
    }
}
