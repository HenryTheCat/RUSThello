//! It features methods to convert a game to strings of text and vice-versa, to interface with external AIs.

use reversi::{Player, Cell, Game, Status, Board, BOARD_SIZE};

/// It converts a given Status to a string:
/// "O" for status Running { Light };
/// "X" for status Running { Dark };
/// "." for status Ended.
pub fn status_to_string(status: Status) -> String {
    match status {
        Status::Running { current_player } => match current_player {
            Player::Light => "O".to_string(),
            Player::Dark => "X".to_string(),
        },
        Status::Ended => ".".to_string(),
    }
}


/// It converts a given board to a string. For every cell, it is inserted:
/// 'O' for a Light disk;
/// 'X' for a Dark disk;
/// '.' for an Empty cell.
pub fn board_to_string(board: Board) -> String {

    let mut board_to_string = String::new();

    for row_array in board.iter() {
        for &cell in row_array.iter() {
            board_to_string.push ( match cell {
                Cell::Empty => '.',
                Cell::Taken { disk } => match disk {
                    Player::Light => 'O',
                    Player::Dark => 'X',
                }
            } );
        }
    }

    board_to_string
}


/// It takes a board and a status string, where the information is encoded the same way as `board_to_string` and `status_to_string` do, and converts them into a Game.
pub fn string_to_game(board_string: String, status_string: String) -> Game {

    let status: Status;
    match status_string.chars().next() {
        Some( player_char ) => {
            status = match player_char {
                'O' => Status::Running { current_player: Player::Light },
                'X' => Status::Running { current_player: Player::Dark },
                '.' => Status::Ended,
                _ => panic!("string_to_game received a invalid char in player_string!"),
            };
        }
        None => panic!("string_to_game does not received a char in player_string!"),
    }

    let mut board: Board = [[Cell::Empty; BOARD_SIZE]; BOARD_SIZE];
    let board_into_bytes = board_string.into_bytes();
    for row in 0..BOARD_SIZE {
        for col in 0..BOARD_SIZE {
            board[row][col] = match (board_into_bytes[ BOARD_SIZE * row + col ]) as char {
                    'O' => Cell::Taken { disk: Player::Light },
                    'X' => Cell::Taken { disk: Player::Dark },
                    '.' => Cell::Empty,
                    _ => panic!("string_to_game received a invalid char in board_string!"),
            }
        }
    };

    Game::new(board, status)
}
