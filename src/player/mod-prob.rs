//! Provides `game::IsPlayer<::OtherAction>` types.

use interface;
use reversi;
use reversi::{board, turn, game};
use reversi::board::{Coord};
use ::{Result, Action};
use std::cmp::Ordering;
use std::thread;
use probability::distribution::{Distribution, Gaussian};

/// The type of human players. Its 'make_move' calls the interface to ask user for an input.
pub struct HumanPlayer;

impl game::IsPlayer<::OtherAction> for HumanPlayer {
    /// Calls the interface to ask user for an input.
    fn make_move(&self, turn: &turn::Turn) -> Result<Action> {
        interface::human_make_move(turn)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Score {
    Running(f64),
    Ended(i16),
}

impl PartialOrd<Score> for Score {
    fn partial_cmp(&self, other: &Score) -> Option<Ordering> {
        let greater = match *self {
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
        };
        if greater {
            Some(Ordering::Greater)
        } else {
            Some(Ordering::Less)
        }
    }
}

// #[derive(Clone)]
pub struct Evaluation(Box<Fn(f64) -> f64>);

impl Distribution for Evaluation {
    type Value = f64;

    fn distribution(&self, up_to: f64) -> f64 {
        self.0(up_to)
    }
}

impl Evaluation {
    pub fn gaussian(mu: f64, sigma: f64) -> Evaluation {
        Evaluation(Box::new(move |up_to| Gaussian::new(mu, sigma).distribution(up_to)))
    }

    pub fn max(evals: Vec<Evaluation>) -> Evaluation {
        Evaluation(Box::new(move |up_to| evals.iter().map( move |eval| eval.distribution(up_to)).product()))
    }

    pub fn not(eval: Evaluation) -> Evaluation {
        Evaluation(Box::new(move |up_to| 1f64 - eval.distribution(up_to) ))
    }

    pub fn min(evals: Vec<Evaluation>) -> Evaluation {
        Evaluation::not( Evaluation(Box::new(move |up_to| evals.iter().map(move |eval| 1f64 - eval.distribution(up_to)).product() )))
    }
}

// #[derive(Clone)]
pub enum DistScore {
    Ended(i16),
    Running(Evaluation),
}

impl DistScore {
    pub fn to_score(&self) -> Score {
        match *self {
            DistScore::Running(ref eval) => Score::Running(board::NUM_CELLS as f64 - eval.distribution(0f64)),
            DistScore::Ended(ref scr) => Score::Ended(*scr),
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
        match *self {
            AiPlayer::Weak   => Ok(game::PlayerAction::Move(try!(AiPlayer::find_best_move(turn, 1)))),
            AiPlayer::Medium => Ok(game::PlayerAction::Move(try!(AiPlayer::find_best_move(turn, 3)))),
            AiPlayer::Strong => Ok(game::PlayerAction::Move(try!(AiPlayer::find_best_move(turn, 5)))),
        }
    }
}

impl AiPlayer {
    /// Find best moves among the legal ones.
    /// Each possibility is evaluated by a method depending on the value of `self` and confronted with the others.
    pub fn find_best_move(turn: &turn::Turn, depth: u8) -> Result<board::Coord> {

        let side = try!(turn.get_state().ok_or(reversi::ReversiError::EndedGame(*turn)));
        let mut best_move: Option<(board::Coord, Score)> = None;
        let mut threadjoins = Vec::new();

        for index in 0..board::NUM_CELLS {
            let coord = try!(board::Coord::from_index(index));
            if let Ok(turn_after_move) = turn.check_and_move(coord) {
                threadjoins.push(thread::spawn(move || Ok((coord, try!(AiPlayer::ai_eval(&turn_after_move, depth)))) ));
            }
        }

        for threadjoin in threadjoins {
            let (new_coord, new_score) = try!(threadjoin.join().expect("Could not receive answer"));
            if let Some(some_best_move) = best_move {
                match side {
                    // scores represent the probability of ending with cell diff < 0!
                    reversi::Side::Dark  if new_score < some_best_move.1 => best_move = Some((new_coord, new_score)),
                    reversi::Side::Light if new_score > some_best_move.1 => best_move = Some((new_coord, new_score)),
                    _ => {},
                }
            } else {
                best_move = Some((new_coord, new_score));
            }
        }

        best_move.ok_or(reversi::ReversiError::EndedGame(*turn)).map(|some_best_move| some_best_move.0)
    }

    /// Converts DistScore to Score
    fn ai_eval(turn: &turn::Turn, depth: u8) -> Result<Score> {
        // Ok(try!(AiPlayer::ai_prob_eval(turn, depth)).to_score()) // alternative equivalent version
        AiPlayer::ai_prob_eval(turn, depth).map(|dist_score| dist_score.to_score())
    }

    fn ai_prob_eval(turn: &turn::Turn, depth: u8) -> Result<DistScore> {
        match *turn.get_state() {
            None => Ok(DistScore::Ended(turn.get_score_diff())),
            Some(side) => {
                if depth == 0 {
                    // let mu: f64 = try!(AiPlayer::heavy_eval(&turn));
                    // let sigma: f64 = 0.5f64 * (board::NUM_CELLS as u8 - turn.get_tempo()) as f64;
                    // let gauss = Evaluation::gaussian(0f64, sigma);
                    // let eval = Evaluation(Box::new(move |up_to| gauss.0(0.5f64 * up_to) + mu));
                    let val = AiPlayer::heavy_eval(&turn).unwrap();
                    let eval = Evaluation(Box::new(move |up_to| {
                        if up_to < val {
                            0f64
                        } else {
                            1f64
                        }
                    }));
                    Ok(DistScore::Running(eval))
                } else {
                    let mut dist_evals: Vec<Evaluation> = Vec::new();
                    let mut best_end_score: Option<i16> = None;
                    for index in 0..board::NUM_CELLS {
                        let coord = try!(board::Coord::from_index(index));
                        if let Ok(turn_after_move) = turn.check_and_move(coord) {
                            match AiPlayer::ai_prob_eval(&turn_after_move, depth - 1).unwrap() {
                                DistScore::Running(new_eval) => dist_evals.push(new_eval),
                                DistScore::Ended(new_score) => {
                                    if let Some(scr) = best_end_score {
                                        match side {
                                            reversi::Side::Dark  if new_score < scr => best_end_score = Some(new_score),
                                            reversi::Side::Light if new_score > scr => best_end_score = Some(new_score),
                                            _ => {},
                                        }
                                    } else {
                                        best_end_score = Some(new_score);
                                    }
                                }
                            }
                        }
                    }
                    if let Some(scr) = best_end_score {
                        match side {
                            reversi::Side::Dark  if scr < 0 || dist_evals.is_empty() => Ok(DistScore::Ended(scr)),
                            reversi::Side::Light if scr > 0 || dist_evals.is_empty() => Ok(DistScore::Ended(scr)),
                            _ => {
                                if !dist_evals.is_empty() {
                                    match side {
                                        reversi::Side::Dark  => Ok(DistScore::Running(Evaluation::min(dist_evals))),
                                        reversi::Side::Light => Ok(DistScore::Running(Evaluation::max(dist_evals))),
                                    }
                                } else {
                                    unreachable!();
                                }
                            }
                        }
                    } else if !dist_evals.is_empty() {
                        match side {
                            reversi::Side::Dark  => Ok(DistScore::Running(Evaluation::min(dist_evals))),
                            reversi::Side::Light => Ok(DistScore::Running(Evaluation::max(dist_evals))),
                        }
                    } else {
                        unreachable!();
                    }
                }
            }
        }
    }

    fn heavy_eval(turn: &turn::Turn) -> Result<f64> {
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
        Ok( board::NUM_CELLS as f64 * (score_light as f64 - score_dark as f64 ) / ( score_dark + score_light) as f64 )
    }
}
