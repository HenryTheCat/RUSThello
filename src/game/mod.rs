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
#[derive(Clone, Copy, PartialEq)]
pub enum Status {
    Running { next_player: Player },
    Ended,
}

/// Each cell in the board can either be empty or taken by one of the players.
#[derive(Clone, Copy, PartialEq)]
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

/// The board is given by a matrix of cells of size BOARD_SIZE and by which player has to move next.
#[derive(Clone, Copy, PartialEq)]
pub struct Game {
    board: [[Cell; BOARD_SIZE]; BOARD_SIZE],
    status: Status,
}

impl Game {
    /// Initializing a new game: starting positions on the board and Dark is the first to play
    pub fn new() -> Game {
        let mut board = [[Cell::Empty; BOARD_SIZE]; BOARD_SIZE];
        board[3][3] = Cell::Taken { player: Player::Light };
        board[4][4] = Cell::Taken { player: Player::Light };
        board[3][4] = Cell::Taken { player: Player::Dark };
        board[4][3] = Cell::Taken { player: Player::Dark };
        Game {
            board: board,
            status: Status::Running { next_player: Player::Dark },
        }
    }

    /// Return the game's board
    pub fn get_board(&self) -> [[Cell; BOARD_SIZE]; BOARD_SIZE] {
        self.board
    }

    /// Return the game's status
    pub fn get_status(&self) -> Status {
        self.status
    }

    /// Check that a given move is legal
    pub fn check_move (&self, (row, col): (usize, usize)) -> bool {

        // If the given coordinate falls out of the board or in a taken cell, the move cannot be legal
        if row >= BOARD_SIZE || col >= BOARD_SIZE || self.board[row][col] != Cell::Empty {
            return false;
        }

        // If a move leads to eat in at least one direction, then it is legal
        for dir in 0..DIRECTIONS {
            if self.check_move_along_direction((row, col),  dir) {
                return true;
            }
        }
                
        false
    }

    // Check whether a move leads to eat in a specified direction
    fn check_move_along_direction (&self, coord: (usize, usize), dir: usize) -> bool {

        let (mut next_row, mut next_col): (usize, usize) = coord;
        let mut eating: bool = false;
        let (delta_ns, delta_ew) = DIRECTION[dir];
        
        loop {
            if     ( next_row == 0 && delta_ns == -1 )
                || ( next_col == 0 && delta_ew == -1 )
                || ( next_row == ( BOARD_SIZE - 1 ) && delta_ns == 1 )
                || ( next_col == ( BOARD_SIZE - 1 ) && delta_ew == 1 ) {
                    return false;
                } else {
                    next_row = (next_row as i8 + delta_ns) as usize;
                    next_col = (next_col as i8 + delta_ew) as usize;
                    match self.board[next_row][next_col] {
                        Cell::Empty => return false,
                        Cell::Taken { player } => {
                            if let Status::Running { next_player } = self.status {
                                if player == next_player {
                                    return eating;
                                } else {
                                    eating = true;
                                }
                            }
                        }
                    }
                }
        }
    }
    
    // Eats all of the opponent's occupied cells from a specified cell (given by its coordinates) in a specified direction
    // until it finds a cell of the current player
    // WARNING: this function do NOT perform any check about whether or not the move is legal
    fn eat_along_direction (&mut self, coord: (usize, usize), dir: usize) {

        //let (delta_row, delta_col) = Game::match_dir(dir);        
        let (mut next_row, mut next_col): (usize, usize) = coord;
        let (delta_ns, delta_ew) = DIRECTION[dir];
        
        loop {
            next_row = (next_row as i8 + delta_ns) as usize;
            next_col = (next_col as i8 + delta_ew) as usize;
            if let Status::Running { next_player } = self.status {
                if self.board[next_row][next_col] == ( Cell::Taken { player: next_player } ) {
                    break;
                } else {
                    self.board[next_row][next_col] = Cell::Taken { player: next_player };
                }
            }
        }
    }

    // Current player perform a move, after verifying that it is legal
    pub fn make_move (&mut self, (row, col): (usize, usize)) -> &Game {
        
        if row >= BOARD_SIZE || col >= BOARD_SIZE || self.board[row][col] != Cell::Empty {
            return self;
        }

        let mut legal: bool = false;

        for dir in 0..DIRECTIONS {
            if self.check_move_along_direction((row, col),  dir) {
                self.eat_along_direction((row, col), dir);
                legal = true;
            }
        }

        // If a move was legal, the next player to play has to be determined
        // If the opposite player can make any move at all, it gets the turn
        // If not, if the previous player can make any move at all, it gets the turn
        // If not (that is, if no player can make any move at all) the game is ended
        if legal {
            if let Status::Running { next_player } = self.status {
                self.board[row][col] = Cell::Taken { player: next_player };

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
        
        self
    }

    // Return whether or not next_player can make any move at all
    fn can_move(&self) -> bool {
        for row in 0..BOARD_SIZE {
            for col in 0..BOARD_SIZE {
                if self.check_move((row, col)) {
                    return true;
                }
            }
        }
        false
    }

    // Return the current score of the match
    pub fn get_score(&self) -> (u8, u8) {
        let (mut score_light, mut score_dark): (u8, u8) = (0, 0);

        for row in 0..BOARD_SIZE {
            for col in 0..BOARD_SIZE {
                match self.board[row][col] {
                    Cell::Taken { player: Player::Light } => score_light += 1,
                    Cell::Taken { player: Player::Dark } => score_dark += 1,
                    _ => {},
                }
            }
        }

        (score_light, score_dark)
    }

    pub fn get_turn(&self) -> u8 {
        let mut turn: u8 = 0;

        for row in 0..BOARD_SIZE {
            for col in 0..BOARD_SIZE {
                if let Cell::Taken { player: _ } =  self.board[row][col] {
                    turn += 1;
                }
            }
        }
        turn
    }

    pub fn get_cell(&self, (row, col): (usize, usize)) -> Cell {
        return self.board[row][col];
    }

}
