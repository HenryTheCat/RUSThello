use interface;
use std::process::Command;
use rusthello_lib::game;
use rusthello_lib;



/// Opponent represent the different kind of player who can take part to the game
pub enum Opponent {
    Human,
    ExternalAI { ai_path: String },
}



impl Opponent {

    /// make_move matches each kind of Opponent with its make_move method
    pub fn make_move(&self, game: &game::Game) -> (usize, usize) {
        match *self {
            Opponent::Human => {
                return interface::human::make_move(game);
            }
            Opponent::ExternalAI { ref ai_path } => {

                let next_player_to_string: String;
                if let game::Status::Running { next_player } = game.get_status() {
                    next_player_to_string = rusthello_lib::status_to_string( game::Status::Running { next_player: next_player } );
                } else {
                    panic!("External AI called on ended game!");
                }

                let board_to_string = rusthello_lib::board_to_string(game.get_board());

                let output = Command::new(ai_path)
                                     .arg(next_player_to_string)
                                     .arg(board_to_string)
                                     .output()
                                     .unwrap_or_else(|e| { panic!("failed to execute process: {}", e) });
                let ai_move = (output.stdout[0] as usize, output.stdout[1] as usize);

                interface::print_move(game, ai_move);

                return ai_move;
            }
        }
    }

}
