//! Bench module.
#![feature(test)]

extern crate reversi;
extern crate rusthello_lib;
extern crate test;

use reversi::game::Game;
use rusthello_lib::ai_player::AiPlayer;

#[bench]
fn test_ai(b: &mut test::Bencher) {

    let adam = AiPlayer::Weak;
    let eve  = AiPlayer::Weak;

    b.iter(|| {
               // First match: Adam is Dark, Eve is Light
               let mut game = Game::new(&adam, &eve);

               while !game.is_endgame() {
                   game.play_turn().expect("`play_turn` returned an error");
               }
           });

    b.iter(|| {
               // Second match: Eve is Dark, Adam is Light
               let mut game = Game::new(&eve, &adam);

               while !game.is_endgame() {
                   game.play_turn().expect("`play_turn` returned an error");
               }
           });

}
