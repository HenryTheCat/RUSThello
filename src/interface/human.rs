// This module implements the input/output needed for a human player

use game;
use std::io::{self, Write};

/// make_move ask get a human player's input and convert it into a move. If the move if illegal, it ask for another input until the given move is a legal one.
pub fn make_move(game: &game::Game) -> (usize, usize) {

    if let game::Status::Running { next_player } = game.get_status() {
        match next_player {
            game::Player::Light => print!("○ Light moves: "),
            game::Player::Dark  => print!("● Dark moves: "),
        }
    }
    let mut next_move = String::new();
    let (mut row, mut col): (usize, usize) = (9, 9);

    loop {

        // Read the input
        let _ = io::stdout().flush();
        io::stdin().read_line(&mut next_move)
            .ok()
            .expect("Failed to read line");

        // Every char in the input string which is a digit (0--9, a(A)--z(Z)) is interpreted as a row or column
        // and assigned to the relevant variable
        for curr_char in next_move.chars() {
            if let Some(digit) = curr_char.to_digit(36) {
                if digit <= 9 {
                    // rows have to be corrected with a -1, as the first row (labelled '1') has index 0 and so on...
                    row = ( digit - 1 ) as usize;
                } else if digit >= 10 {
                    // rows have to be corrected with a -10, as the first column (labelled 'a') has index 0 and so on...
                    col = ( digit - 10 ) as usize;
                }
            }
            
        }

        // If the given move is valid, return it; otherwise complain about the illegal move and ask for another one
        if game.check_move((row, col)) {
            return (row, col);
        } else {
            print!("Illegal move, try again: ");
        }
    }
}
