use reversi;
use reversi::{board, turn, game};
use reversi::board::Coord;
use {Result, Action, Side};
use std::cmp::Ordering;
use rand;
use rand::distributions::{IndependentSample, Range};
use rayon::prelude::*;

const RANDOMNESS: f64 = 0.05f64;
const STRENGHT: u32 = 100;

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
                              Score::Ended(scr2) => scr2 < 0i16 || (scr2 == 0i16 && val1 > 0f64),
                          }
                      }
                      Score::Ended(scr1) => {
                          match *other {
                              Score::Running(val2) => scr1 > 0i16 || (scr1 == 0i16 && val2 < 0f64),
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

impl Eq for Score {}

impl Ord for Score {
    fn cmp(&self, other: &Score) -> Ordering {
        self.partial_cmp(other).expect("Should be ordered")
    }
}

pub struct ExpAiPlayer {}

impl game::IsPlayer<::OtherAction> for ExpAiPlayer {
    /// Calls `find_best_move` with suitable parameters
    fn make_move(&self, turn: &turn::Turn) -> Result<Action> {
        Ok(game::PlayerAction::Move(try!(ExpAiPlayer::find_best_move(turn, STRENGHT))))
    }
}

impl ExpAiPlayer {
    /// Find best moves among the legal ones.
    /// Each possibility is evaluated by a method depending on the value of `self` and confronted with the others.
    pub fn find_best_move(turn: &turn::Turn, comps: u32) -> Result<board::Coord> {

        // If everything is alright, turn shouldn't be ended
        let side = turn.get_state()
            .ok_or(reversi::ReversiError::EndedGame(*turn))?;

        // Finds all possible legal moves and records their coordinates
        let mut moves: Vec<Coord> = Vec::new();
        for row in 0..board::BOARD_SIZE {
            for col in 0..board::BOARD_SIZE {
                let coord = board::Coord::new(row, col);
                if turn.check_move(coord).is_ok() {
                    moves.push(coord);
                }
            }
        }

        match moves.len() {
            0 => unreachable!("Game is not ended!"), // Game can't be ended
            1 => Ok(moves[0]), // If there is only one possible move, there's no point in evaluating it.
            num_moves @ _ => {
                // Each move has to be evaluated in order to find the best one
                let moves_and_scores = moves
                    .par_iter()
                    .map(|&coord| {
                        let mut turn_after_move = *turn;
                        turn_after_move
                            .make_move(coord)
                            .expect("The move was checked, but something went wrong!");
                        let score = ExpAiPlayer::ai_eval(&turn_after_move, comps / num_moves as u32)
                            .expect("Something went wrong with `AiPlayer::ai_eval`!");
                        (coord, score)
                    });
                let best_move_and_score = match side {
                        Side::Dark => moves_and_scores.min_by_key(|&(_, score)| score),
                        Side::Light => moves_and_scores.max_by_key(|&(_, score)| score),
                    }
                    .expect("No best move found!");
                Ok(best_move_and_score.0)
            }
        }
    }

    fn ai_eval(turn: &turn::Turn, comps: u32) -> Result<Score> {
        if turn.is_end_state() {
            Ok(Score::Ended(turn.get_score_diff()))
        } else {
            let mut score = try!(ExpAiPlayer::ai_eval_with_leftover(turn, comps)).0;
            // Add some randomness
            let between = Range::new(-RANDOMNESS, RANDOMNESS);
            let mut rng = rand::thread_rng();
            score = match score {
                Score::Running(val) => Score::Running(val * (1.0 + between.ind_sample(&mut rng))),
                _ => score,
            };
            // Done, return
            Ok(score)
        }
    }

    fn ai_eval_with_leftover(turn: &turn::Turn, comps: u32) -> Result<(Score, u32)> {

        // If everything is alright, turn shouldn't be ended
        // assert!(!this_turn.is_endgame());

        // Finds all possible legal moves and records their coordinates
        let mut moves: Vec<Coord>;
        let mut turn = turn.clone();
        loop {
            moves = Vec::new();
            for row in 0..board::BOARD_SIZE {
                for col in 0..board::BOARD_SIZE {
                    let coord = board::Coord::new(row, col);
                    if turn.check_move(coord).is_ok() {
                        moves.push(coord);
                    }
                }
            }
            match moves.len() {
                0 => unreachable!("Endgame should have been detected earlier: here it's a waste of computations!"),
                1 => {
                    turn.make_move(moves[0])
                        .expect("There is one move and it should be legit");
                    if turn.is_end_state() {
                        return Ok((Score::Ended(turn.get_score_diff()), comps));
                    }
                }
                _ => break,
            }
        }

        // If everything is alright, turn shouldn't be ended
        // assert!(!turn.is_endgame());

        let mut scores: Vec<Score> = Vec::new();
        let mut leftover = comps.checked_sub(moves.len() as u32).unwrap_or(0);
        //let certainty: f64 = ( turn.get_tempo() as f64 + (1f64 + leftover as f64).ln() ) / 8f64;

        while let Some(coord) = moves.pop() {
            let mut turn_after_move = turn.clone();
            turn_after_move.make_move(coord)?;
            let turns_left = (moves.len() + 1) as u32;
            scores.push(match turn_after_move.get_state() {
                            None => Score::Ended(turn_after_move.get_score_diff()),
                            Some(_) if leftover < turns_left => Score::Running(try!(ExpAiPlayer::heavy_eval(&turn_after_move))),
                            _ => {
                                let new_comps = leftover / turns_left; // since leftover >= turns_left, then new_comps >= 1
                                let new_score_leftover = ExpAiPlayer::ai_eval_with_leftover(&turn_after_move, new_comps)?;
                                leftover += new_score_leftover.1;
                                leftover -= new_comps; // since leftover >= turns_left, leftover - newcomps >= 0
                                new_score_leftover.0
                            }
                        });
        }

        //let mut scr: f64 = 0f64;
        //let mut val: i16 = 0;
        //match turn.get_state() {
        //    reversi::Side::Dark => {
        //        scores.into_iter().map(|score| {
        //            if let Score::Running(new_scr) = score {
        //                scr += new_scr.recip().powf(certainty);
        //            }
        //        });
        //        scr.powf(certainty.recip()).recip();
        //    }
        //    reversi::Side::Light => {
        //        scores.into_iter().map(|score| {
        //            if let Score::Running(new_scr) = score {
        //                scr += new_scr.powf(certainty);
        //            }
        //        });
        //        scr.powf(certainty.recip());
        //    }
        //    _ => panic!("should not happen!"),
        //}
        //
        //Ok((Score::Running(scr), leftover))

        Ok((match turn.get_state() {
                Some(reversi::Side::Dark) => scores.into_iter().min().expect("Why should this fail?"),
                Some(reversi::Side::Light) => scores.into_iter().max().expect("Why should this fail?"),
                None => unreachable!("turn is ended but it should not be"),
            },
            leftover))
    }

    fn heavy_eval(turn: &turn::Turn) -> Result<f64> {
        // Weights
        const CORNER_BONUS: u16 = 50;
        const ODD_CORNER_MALUS: u16 = 20;
        const EVEN_CORNER_BONUS: u16 = 10;
        const ODD_MALUS: u16 = 7; // x2
        const EVEN_BONUS: u16 = 3; // x2
        // ------------------------ Sum = 100

        // Special cells
        let sides: [(Coord, Coord, Coord, Coord, Coord, Coord, Coord); 4] = [(/* NW corner */
                                                                              Coord::new(0, 0),
                                                                              Coord::new(0, 1),
                                                                              Coord::new(1, 1),
                                                                              Coord::new(0, 2),
                                                                              Coord::new(2, 2),
                                                                              Coord::new(1, 0),
                                                                              Coord::new(2, 0)),
                                                                             (/* NE corner */
                                                                              Coord::new(0, 7),
                                                                              Coord::new(1, 7),
                                                                              Coord::new(1, 6),
                                                                              Coord::new(2, 7),
                                                                              Coord::new(2, 5),
                                                                              Coord::new(0, 6),
                                                                              Coord::new(0, 5)),
                                                                             (/* SW corner */
                                                                              Coord::new(7, 0),
                                                                              Coord::new(6, 0),
                                                                              Coord::new(6, 1),
                                                                              Coord::new(5, 0),
                                                                              Coord::new(5, 2),
                                                                              Coord::new(7, 1),
                                                                              Coord::new(7, 2)),
                                                                             (/* SE corner */
                                                                              Coord::new(7, 7),
                                                                              Coord::new(6, 7),
                                                                              Coord::new(6, 6),
                                                                              Coord::new(5, 7),
                                                                              Coord::new(5, 5),
                                                                              Coord::new(7, 6),
                                                                              Coord::new(7, 5))];

        let mut score_light: u16 = 1;
        let mut score_dark: u16 = 1;

        for &(corner, odd, odd_corner, even, even_corner, counter_odd, counter_even) in &sides {

            if let Some(disk) = *turn.get_cell(corner)? {
                match disk.get_side() {
                    Side::Light => score_light += CORNER_BONUS,
                    Side::Dark => score_dark += CORNER_BONUS,
                }
            } else {
                for &(coord_odd, coord_even) in &[(odd, even), (counter_odd, counter_even)] {
                    if let Some(disk) = *turn.get_cell(coord_odd)? {
                        match disk.get_side() {
                            Side::Light => score_dark += ODD_MALUS,
                            Side::Dark => score_light += ODD_MALUS,
                        }
                    } else if let Some(disk) = *turn.get_cell(coord_even)? {
                        match disk.get_side() {
                            Side::Light => score_light += EVEN_BONUS,
                            Side::Dark => score_dark += EVEN_BONUS,
                        }
                    }
                }
                if let Some(disk) = *turn.get_cell(odd_corner)? {
                    match disk.get_side() {
                        Side::Light => score_dark += ODD_CORNER_MALUS,
                        Side::Dark => score_light += ODD_CORNER_MALUS,
                    }

                } else if let Some(disk) = *turn.get_cell(even_corner)? {
                    match disk.get_side() {
                        Side::Light => score_light += EVEN_CORNER_BONUS,
                        Side::Dark => score_dark += EVEN_CORNER_BONUS,
                    }
                }
            }
        }
        Ok(score_light as f64 - score_dark as f64)
    }
}
