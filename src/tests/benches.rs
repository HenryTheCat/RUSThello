//! Bench module.

use test;
use reversi::game;

#[bench]
fn test_ai(b: &mut test::Bencher) {

    let adam = ::ai_player::AiPlayer::Weak;
    let eve = ::ai_player::AiPlayer::Weak;

    b.iter(|| {
               // First match: Adam is Dark, Eve is Light
               let mut game = game::Game::new(&adam, &eve);

               while !game.is_endgame() {
                   game.play_turn().expect("`play_turn` returned an error");
               }
           });

    b.iter(|| {
               // Second match: Eve is Dark, Adam is Light
               let mut game = game::Game::new(&eve, &adam);

               while !game.is_endgame() {
                   game.play_turn().expect("`play_turn` returned an error");
               }
           });

}
