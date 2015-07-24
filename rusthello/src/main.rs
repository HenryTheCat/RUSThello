// Implementation in Rust of a simple Reversi game
// v 0.2.0
// by Enrico Ghiorzi

// Import modules
mod interface;
mod opponents;

extern crate rusthello_lib;
use rusthello_lib::game;



fn main() {

    // Create a new game and two opponents

    let (light, dark) = interface::start_game();

    let mut game = game::Game::new();

    // Proceed with turn after turn till the endgame
    loop {
        // Draw the current board and game info
        interface::draw_board(&game);

        // Depending on the status of the game, proceed with the next turn or declare the winner
        match game.get_status() {

            // If the game is running, get the coordinates of the new move from the next player
            game::Status::Running { next_player } => {

                let coord: (usize, usize);
                match next_player {
                    game::Player::Light => coord = light.make_move(&game),
                    game::Player::Dark  => coord = dark.make_move(&game),
                }

                // If the new move is valid, perform it; otherwise panic
                // NB: opponents's make_move method is responsible for returning a legal move
                //     so the program should never print this message unless something goes horribly wrong
                if game.check_move(coord) {
                    game.make_move(coord);
                } else {
                    panic!("Invalid move sent to main!");
                }
            }

            // If the game is ended, exit the loop
            game::Status::Ended => {
                break;
            }
        }
    }
}
