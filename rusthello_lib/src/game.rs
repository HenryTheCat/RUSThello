// This module provides the main structures and mechanics of the game

/// There are two players playing the match: Light and Dark
#[derive(Clone, Copy, PartialEq)]
pub enum Player {
    Light,
    Dark,
}

impl Player {
    /// Get the player of the opposite kind to self
    fn opposite(&self) -> Player {
        match *self {
            Player::Light => {
                Player::Dark
                    }
            Player::Dark => {
                Player::Light
            }
        }
    }
}

/// A game can be in two status: either running (with a next player to play) or ended.
#[derive(Clone)]
pub enum Status {
    Running { next_player: Player },
    Ended,
}

/// Each cell in the board can either be empty or taken by one of the players.
#[derive(Clone, Copy)]
pub enum Cell {
    Empty,
    Taken { player: Player },
}

/// There are eight cardinal directions
const DIRECTIONS: usize = 8;

/// An array listing all the cardinal directions, represented by the coordinate delta to move in that direction
/// Example: if I am in cell (4, 5) and move NE, I go to cell (4, 5) + (1, -1) = (5, 4)
const DIRECTION: [(i8, i8); DIRECTIONS] = [
    ( 0, -1), //North
    ( 1, -1), //NE
    ( 1,  0), //East
    ( 1,  1), //SE
    ( 0,  1), //South
    (-1,  1), //SW
    (-1,  0), //West
    (-1, -1), //NW
    ];

/// The size of the board is a constant.
pub const BOARD_SIZE: usize = 8;

/// Board is the type of boards, that is, bidimensional arrays of Cells of size BOARD_SIZE.
pub type Board = [[Cell; BOARD_SIZE]; BOARD_SIZE];

/// The board is given by a matrix of cells of size BOARD_SIZE and by which player has to move next.
#[derive(Clone)]
pub struct Game {
    board: Board,
    status: Status,
    score_light: u8,
    score_dark: u8,
}

impl Game {
    /// Initializing a new game: starting positions on the board and Dark is the first to play
    pub fn new() -> Game {
        let mut board: Board = [[Cell::Empty; BOARD_SIZE]; BOARD_SIZE];
        board[3][3] = Cell::Taken { player: Player::Light };
        board[4][4] = Cell::Taken { player: Player::Light };
        board[3][4] = Cell::Taken { player: Player::Dark };
        board[4][3] = Cell::Taken { player: Player::Dark };

        Game {
            board: board,
            status: Status::Running { next_player: Player::Dark },
            score_light: 2,
            score_dark: 2,
        }
    }

    /// Initializing a new game given status and board
    pub fn new_custom_game(status: Status, board: Board) -> Game {

        let mut score_light: u8 = 0;
        let mut score_dark: u8 = 0;
        for row_array in board.iter() {
            for &cell in row_array.iter() {
                match cell {
                    Cell::Taken { player } => {
                        match player {
                            Player::Light => score_light += 1,
                            Player::Dark => score_dark += 1,
                        }
                    }
                    _ => {}
                }
            }
        }

        Game {
            board: board,
            status: status,
            score_light: score_light,
            score_dark: score_dark,
        }
    }

    /// Return the game's board
    pub fn get_board(&self) -> Board {
        self.board
    }

    /// Return the game's status
    pub fn get_status(&self) -> Status {
        self.status.clone()
    }

    /// Check that a given move is legal
    pub fn check_move (&self, (row, col): (usize, usize)) -> bool {

        // If the given coordinate falls out of the board or in a taken cell, the move cannot be legal
        if row >= BOARD_SIZE || col >= BOARD_SIZE {
            return false;
        } else if let Cell::Taken { .. } = self.board[row][col] {
            return false;
        }

        // If a move leads to eat in at least one direction, then it is legal
        for &dir in &DIRECTION {
            if self.check_move_along_direction((row, col), dir) {
                return true;
            }
        }

        false
    }



    // Check whether a move leads to eat in a specified direction
    // Does NOT perform checks already performed by check_move!
    fn check_move_along_direction (&self, (row, col): (usize, usize), (delta_ns, delta_ew): (i8, i8)) -> bool {

        // Need at least two cells' space in the given direction
        let mut col_i8: i8 = col as i8 + 2*delta_ew;
        if ( col_i8 < 0 ) || ( col_i8 >= BOARD_SIZE as i8 ) {
                return false;
        }

        let mut row_i8: i8 = row as i8 + 2*delta_ns;
        if ( row_i8 < 0 ) || ( row_i8 >= BOARD_SIZE as i8 ) {
                return false;
        }

        if let Status::Running { next_player } = self.status {

            // Next cell has to be owned by the other player
            if let Cell::Taken { player } = self.board[ ( row as i8 + delta_ns ) as usize ][ ( col as i8 + delta_ew ) as usize] {
                if player == next_player {
                    return false;
                }

                while let Cell::Taken { player } = self.board[ row_i8 as usize ][ col_i8 as usize] {
                    if player == next_player {
                        return true;
                    }

                    col_i8 += delta_ew;
                    if ( col_i8 < 0 ) || ( col_i8 >= BOARD_SIZE as i8 ) {
                            return false;
                    }

                    row_i8 += delta_ns;
                    if ( row_i8 < 0 ) || ( row_i8 >= BOARD_SIZE as i8 ) {
                            return false;
                    }

                }
            }
        }

        false
    }


    // Eats all of the opponent's occupied cells from a specified cell (given by its coordinates) in a specified direction
    // until it finds a cell of the current player
    // WARNING: this function do NOT perform any check about whether or not the move is legal
    fn eat_along_direction (&mut self, (row, col): (usize, usize), (delta_ns, delta_ew): (i8, i8)) {

        if let Status::Running { next_player } = self.status {

            self.board[ ( row as i8 + delta_ns ) as usize ][ ( col as i8 + delta_ew ) as usize] = Cell::Taken { player: next_player };

            let (mut row_i8, mut col_i8): (i8, i8) = (row as i8 + 2*delta_ns, col as i8 + 2*delta_ew);

            let mut eating: u8 = 1;

            while let Cell::Taken { player: player_in_cell } = self.board[ row_i8 as usize ][ col_i8 as usize] {
                if next_player == player_in_cell {
                    break;
                }

                self.board[ row_i8 as usize ][ col_i8 as usize ] = Cell::Taken { player: next_player };

                eating += 1;
                row_i8 += delta_ns;
                col_i8 += delta_ew;
            }

            match next_player {
                Player::Light => {
                    self.score_light += eating;
                    self.score_dark -= eating;
                }
                Player::Dark => {
                    self.score_light -= eating;
                    self.score_dark += eating;
                }
            }
        }
    }


    // Current player perform a move, after verifying that it is legal
    pub fn make_move (&mut self, (row, col): (usize, usize)) -> &Game {

        if row >= BOARD_SIZE || col >= BOARD_SIZE {
            return self;
        } else if let Cell::Taken { .. } = self.board[row][col] {
            return self;
        }

        let mut legal: bool = false;

        for &dir in &DIRECTION {
            if self.check_move_along_direction((row, col), dir) {
                self.eat_along_direction((row, col), dir);
                    legal = true;
            }
        }

        // If a move is legal, the next player to play has to be determined
        // If the opposite player can make any move at all, it gets the turn
        // If not, if the previous player can make any move at all, it gets the turn
        // If not (that is, if no player can make any move at all) the game is ended
        if legal {
            if self.get_turn() == BOARD_SIZE as u8 * BOARD_SIZE as u8 {
                self.status = Status::Ended;
                return self;
            } else {
                if let Status::Running { next_player } = self.status {
                    self.board[row][col] = Cell::Taken { player: next_player };

                    match next_player {
                        Player::Light => self.score_light += 1,
                        Player::Dark => self.score_dark += 1,
                    }

                    self.status = Status::Running { next_player: next_player.opposite() };
                    if self.can_move() {
                        return self;
                    }

                    self.status = Status::Running { next_player: next_player };
                    if self.can_move() {
                        return self;
                    }

                    self.status = Status::Ended;
                    return self;
                }
            }
        }

        self
    }

    // Return whether or not next_player can make any move at all
    fn can_move(&self) -> bool {

        for (row_n, row) in self.board.iter().enumerate() {
            for (col_n, _cell) in row.iter().enumerate() {
               if self.check_move((row_n, col_n)) {
                    return true;
                }
            }
        }
        false

    }

    // Return the current score of the match
    pub fn get_score(&self) -> (u8, u8) {
        (self.score_light, self.score_dark)
    }

    // Return the difference in score between Light and Dark, in a time-efficient way
    pub fn get_score_diff(&self) -> i16 {
        self.score_light as i16 - self.score_dark as i16
    }

    pub fn get_turn(&self) -> u8 {
        self.score_light + self.score_dark
    }

    pub fn get_cell(&self, (row, col): (usize, usize)) -> Cell {
        self.board[row][col]
    }

}
