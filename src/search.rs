// int negaMax( int depth ) {
//     if ( depth == 0 ) return evaluate();
//     int max = -oo;
//     for ( all moves)  {
//         score = -negaMax( depth - 1 );
//         if( score > max )
//             max = score;
//     }
//     return max;
// }

use crate::{
    movegen::{Move, get_move_string, is_off_board, is_square_attacked},
    moveordering::order_moves,
    piece::*,
    position::Position,
};
use core::panic;

const MATERIAL_PAWN: i32 = 100;
const MATERIAL_KNIGHT: i32 = 320;
const MATERIAL_BISHOP: i32 = 330;
const MATERIAL_ROOK: i32 = 500;
const MATERIAL_QUEEN: i32 = 900;
const MATERIAL_KING: i32 = 20000;

pub fn get_material_score(piece: u8) -> i32 {
    match get_piece_type(piece) {
        PAWN => MATERIAL_PAWN,
        KNIGHT => MATERIAL_KNIGHT,
        BISHOP => MATERIAL_BISHOP,
        ROOK => MATERIAL_ROOK,
        QUEEN => MATERIAL_QUEEN,
        KING => MATERIAL_KING,
        EMPTY => 0,
        _ => panic!("{}", get_piece_type(piece)),
    }
}

fn get_piece_material_score(piece: u8) -> i32 {
    let side = match get_piece_color(piece) {
        WHITE => 1,
        BLACK => -1,
        EMPTY => 0,
        _ => panic!("{}", get_piece_color(piece)),
    };
    let material_score = get_material_score(piece);
    side * material_score
}

pub fn evaluate(position: &Position) -> i32 {
    let mut score = 0;
    let side = match position.is_white_turn {
        true => 1,
        false => -1,
    };
    for square in 0..128 {
        if is_off_board(square) {
            continue;
        }
        let piece = position.board[square];
        score += side * get_piece_material_score(piece);
    }
    score
}

fn alphabeta(
    position: &mut Position,
    mut alpha: i32,
    beta: i32,
    depth: u32,
    nodecount: &mut u64,
) -> i32 {
    if depth == 0 {
        *nodecount += 1;
        return evaluate(position);
    }
    let mut best_value = -100000;
    let moves = position.generate_legal_moves();
    if moves.is_empty() {
        let idx = match position.is_white_turn {
            true => 0,
            false => 1,
        };
        if is_square_attacked(position.king_squares[idx], position) {
            //println!("Checkmate!");
            return -50000;
        } else {
            //println!("Stalemate!");
            return 0;
        }
    }
    // Move ordering
    let moves = order_moves(&position, moves);
    for move_ in moves {
        let piece_at_target = position.board[move_.to];
        let original_castling_rights = position.castling_rights;
        let original_king_squares = position.king_squares;
        let original_ep_square = position.enpassant_square;
        position.make_move(&move_);
        let value = -alphabeta(position, -beta, -alpha, depth - 1, nodecount);
        position.unmake_move(
            &move_,
            piece_at_target,
            original_castling_rights,
            original_king_squares,
            original_ep_square,
        );
        if value > best_value {
            best_value = value;
            if value > alpha {
                alpha = value;
            }
        }
        if value >= beta {
            return best_value;
        }
    }
    best_value
}

pub fn search(position: &mut Position, depth: u32, nodecount: &mut u64) -> Move {
    let mut final_best_move = None;
    for d in 1..depth + 1 {
        let moves = position.generate_legal_moves();
        let mut best_move = None;
        let mut best_value = -100000;
        let mut alpha = -100000;
        let beta = 100000;
        // Move ordering
        let moves = order_moves(&position, moves);
        for move_ in moves {
            let piece_at_target = position.board[move_.to];
            let original_castling_rights = position.castling_rights;
            let original_king_squares = position.king_squares;
            let original_ep_square = position.enpassant_square;
            position.make_move(&move_);
            let value = -alphabeta(position, -beta, -alpha, d, nodecount);
            position.unmake_move(
                &move_,
                piece_at_target,
                original_castling_rights,
                original_king_squares,
                original_ep_square,
            );
            if value > best_value {
                best_move = Some(move_);
                best_value = value;
                if value > alpha {
                    alpha = value;
                }
            }
            if value >= beta {
                //println!("{}", best_value);
                final_best_move = best_move;
            }
        }
        println!(
            "info score cp {best_value} depth {d} nodes {nodecount} pv {}",
            get_move_string(&best_move.unwrap())
        );
        final_best_move = best_move;
    }
    final_best_move.expect("Move is None")
}
