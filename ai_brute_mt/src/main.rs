extern crate rusthello_lib;

use std::env;
use std::str;

mod ai_brute_mt;

fn main() {

    let args: Vec<String> = env::args().collect();

    let status_string = args[1].clone().trim().to_string();
    let board_string = args[2].clone().trim().to_string();
    let game = rusthello_lib::string_to_game(status_string, board_string);
    let (row, col) = ai_brute_mt::make_move(&game);
    match str::from_utf8(&[row as u8, col as u8]).ok() {
        Some(output) => print!("{}", output),
        None => panic!("ai_brute_mt could not send a move!"),
    }
}
