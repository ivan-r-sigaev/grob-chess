// use pico_chess::move_generation::*;
// use pico_chess::game_state::*;

// pub fn print_moves(game_state: &mut GameState, move_list: &mut MoveList) {
//     move_list.generate_moves(game_state);

//     let mut has_moves = false;

//     while let Some(next_move) = move_list.pop_move() {
//         let unmove = game_state.make_move(next_move);
//         if game_state.get_board().is_king_in_check(!game_state.get_turn()) { 
//             game_state.unmake_move(unmove);
//             continue;
//         }

//         has_moves = true;
//         //println!("{:?}", position.board);
//         println!("{:?}", next_move);
//         println!("{}", format_position(game_state));
//         game_state.unmake_move(unmove);
//         std::io::stdin().read_line(&mut String::new()).unwrap();
//     }

//     if !has_moves {
//         println!("No moves found");
//     }
//     move_list.remove_group();
// }

fn main() {
    // const INITIAL_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    // const KIWIPETE_FEN: &str = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
    // let mut position = Position::try_from_fen(KIWIPETE_FEN).unwrap();
    // println!("{}", format_position(&position));
    // print_moves(&mut position);
    // println!("{:?}", position.board)
}
