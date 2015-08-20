use interface;
use std::process::Command;
use rusthello_lib::reversi;
use rusthello_lib::ai_interface;



/// It represents the different kind of player who can take part to the game.
pub enum Opponent {
    Human,
    ExternalAI { ai_path: String },
}



impl Opponent {

    /// It produces the new move from each kind of Opponent.
    pub fn make_move(&self, game: &reversi::Game) -> (usize, usize) {
        match *self {
            Opponent::Human => {
                return interface::human_make_move(game);
            }
            Opponent::ExternalAI { ref ai_path } => {

                let current_player_to_string: String;
                if let reversi::Status::Running { current_player } = game.get_status() {
                    current_player_to_string = ai_interface::status_to_string( reversi::Status::Running { current_player: current_player } );
                } else {
                    panic!("External AI called on ended game!");
                }

                let board_to_string = ai_interface::board_to_string(game.get_board());

                let output = Command::new(ai_path)
                                    .arg(board_to_string)
                                    .arg(current_player_to_string)
                                    .output()
                                    .unwrap_or_else(|e| { panic!("failed to execute process: {}", e) });
                let ai_move = (output.stdout[0] as usize, output.stdout[1] as usize);

                interface::print_move(game, ai_move);

                return ai_move;
            }
        }
    }
}
