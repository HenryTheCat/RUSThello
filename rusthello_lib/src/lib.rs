pub mod game;



pub fn status_to_string(status: game::Status) -> String {
    match status {
        game::Status::Running { next_player } => match next_player {
            game::Player::Light => "O".to_string(),
            game::Player::Dark => "X".to_string(),
        },
        game::Status::Ended => ".".to_string(),
    }
}



pub fn board_to_string(board: game::Board) -> String {

    let mut board_to_string = String::new();

    for row_array in board.iter() {
        for &cell in row_array.iter() {
            board_to_string.push ( match cell {
                game::Cell::Empty => '.',
                game::Cell::Taken { player } => match player {
                    game::Player::Light => 'O',
                    game::Player::Dark => 'X',
                }
            } );
        }
    }

    board_to_string
}



pub fn string_to_game(status_string: String, board_string: String) -> game::Game {

    let status: game::Status;
    match status_string.chars().next() {
        Some( player_char ) => {
            status = match player_char {
                'O' => game::Status::Running { next_player: game::Player::Light },
                'X' => game::Status::Running { next_player: game::Player::Dark },
                '.' => game::Status::Ended,
                _ => panic!("string_to_game received a invalid char in player_string!"),
            };
        }
        None => panic!("string_to_game does not received a char in player_string!"),
    }

    let mut board: game::Board = [[game::Cell::Empty; game::BOARD_SIZE]; game::BOARD_SIZE];
    let board_into_bytes = board_string.into_bytes();
    for row in 0..game::BOARD_SIZE {
        for col in 0..game::BOARD_SIZE {
            board[ row ][ col ] = match (board_into_bytes[ game::BOARD_SIZE * row + col ]) as char {
                    'O' => game::Cell::Taken { player: game::Player::Light },
                    'X' => game::Cell::Taken { player: game::Player::Dark },
                    '.' => game::Cell::Empty,
                    _ => panic!("string_to_game received a invalid char in board_string!"),
            }
        }
    };

    game::Game::new_custom_game(status, board)
}
