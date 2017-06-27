//! Test module.

extern crate rand;
extern crate rayon;
extern crate reversi;
extern crate rusthello_lib;

use reversi::game::Game;
use rusthello_lib::ai_player::AiPlayer;
use std::cmp::Ordering;

mod test_ai;

#[test]
fn test_ai() {

    let adam = AiPlayer::Weak;
    let mut adam_wins = 0;
    let mut adam_total_score: u64 = 0;

    let eve = test_ai::ExpAiPlayer{};
    let mut eve_wins = 0;
    let mut eve_total_score: u64 = 0;

    let mut ties = 0;

    for _ in 0..50 {

        // First match: Adam is Dark, Eve is Light
        let mut game = Game::new(&adam, &eve);

        while !game.is_endgame() {
            game.play_turn().expect("`play_turn` returned an error");
        }

        let (adam_score, eve_score) = game.get_current_score();
        match adam_score.cmp(&eve_score) {
            Ordering::Greater => adam_wins += 1,
            Ordering::Less => eve_wins += 1,
            Ordering::Equal => ties += 1,
        }
        adam_total_score += adam_score as u64;
        eve_total_score += eve_score as u64;

        // Second match: Eve is Dark, Adam is Light
        let mut game = Game::new(&eve, &adam);

        while !game.is_endgame() {
            game.play_turn().expect("`play_turn` returned an error");
        }

        let (eve_score, adam_score) = game.get_current_score();
        match adam_score.cmp(&eve_score) {
            Ordering::Greater => adam_wins += 1,
            Ordering::Less => eve_wins += 1,
            Ordering::Equal => ties += 1,
        }
        adam_total_score += adam_score as u64;
        eve_total_score += eve_score as u64;
    }

    println!("\nAdam wins {} games with total score {}",
             adam_wins,
             adam_total_score);
    println!("Eve wins {} games with total score {}",
             eve_wins,
             eve_total_score);
    println!("Tied {} games\n", ties);
}
