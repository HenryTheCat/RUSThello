use game;
use interface;

pub mod ai_heavy;
//mod ai_brute;
//mod ai_brute_mt;
//mod ai_mean;



/// Opponent represent the different kind of player who can take part to the game
//#[derive(Clone)]
pub enum Opponent {
    Human,
    //AIBrute,
    //AIBruteMT,
    AIHeavy { opponent: ai_heavy::AIHeavy },
    //AIMean,
}



impl Opponent {

    /// make_move matches each kind of Opponent with its make_move method
    pub fn make_move(&mut self, game: &game::Game) -> (usize, usize) {
        match *self {
            Opponent::Human => {
                return interface::human::make_move(game);
            }
            Opponent::AIHeavy { ref mut opponent } => {
                let ai_move = opponent.make_move(game);
                interface::print_move(game, ai_move);
                return ai_move;
            }
            /*
            Opponent::AIBrute => {
                let ai_move = ai_brute::make_move(game);
                interface::print_move(game, ai_move);
                return ai_move;
            }
            Opponent::AIBruteMT => {
                let ai_move = ai_brute_mt::make_move(game);
                interface::print_move(game, ai_move);
                return ai_move;
            }
            Opponent::AIMean => {
                let ai_move = ai_mean::make_move(game);
                interface::print_move(game, ai_move);
                return ai_move;
            }
            */
        }
    }
}
