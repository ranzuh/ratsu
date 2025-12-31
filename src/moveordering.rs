use std::cmp::Reverse;

use crate::{
    evaluation::get_material_score,
    movegen::Move,
    piece::{EMPTY, get_piece_type},
    position::Position,
};

pub fn order_moves(pos: &Position, moves: Vec<Move>) -> Vec<Move> {
    let mut move_scores: Vec<u32> = Vec::with_capacity(100);
    for move_ in &moves {
        let mut score = 0;
        let piece = pos.board[move_.from];
        let target_piece = pos.board[move_.to];

        // score most valuable victim and least valuable attacker (MVV-LVA)
        if get_piece_type(target_piece) != EMPTY {
            score = 10 * get_material_score(target_piece) - get_material_score(piece);
        }
        // TODO: Could also prioritise promotions in future?
        move_scores.push(score as u32);
    }

    // zip the moves + move scores into pairs
    let mut pairs: Vec<_> = moves.into_iter().zip(move_scores.into_iter()).collect();
    // sort pairs by move score
    pairs.sort_by_key(|pair| Reverse(pair.1));
    // unzip to get the sorted moves
    let (moves, _move_scores): (Vec<_>, Vec<_>) = pairs.into_iter().unzip();
    // return sorted moves
    moves
}
