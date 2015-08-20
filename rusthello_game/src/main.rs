//! A simple Reversi game written in Rust with love
//! by Enrico Ghiorzi

// Import modules
mod interface;
mod opponents;

extern crate rusthello_lib;
use rusthello_lib::reversi;



fn main() {

    // Create a new game and two opponents

    let (light, dark) = interface::start_game();

    let mut game = reversi::Game::new_reversi_game();

    // Proceed with turn after turn till the endgame
    loop {
        // Draw the current board and game info
        interface::draw_board(&game);

        // Depending on the status of the game, proceed with the next turn or declare the winner
        match game.get_status() {

            // If the game is running, get the coordinates of the new move from the next player
            reversi::Status::Running { current_player } => {

                let coord: (usize, usize);
                match current_player {
                    reversi::Player::Light => coord = light.make_move(&game),
                    reversi::Player::Dark  => coord = dark.make_move(&game),
                }

                // If the new move is valid, perform it; otherwise panic
                // NB: opponents's make_move method is responsible for returning a legal move
                //     so the program should never print this message unless something goes horribly wrong
                if !game.make_move(coord) {
                    panic!("Invalid move sent to main!");
                }
            }

            // If the game is ended, exit the loop
            reversi::Status::Ended => {
                break;
            }
        }
    }
}
