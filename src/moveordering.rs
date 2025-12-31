use crate::{
    evaluation::get_material_score,
    movegen::Move,
    piece::{EMPTY, get_piece_type},
    position::Position,
};

pub fn order_moves_inplace(pos: &Position, moves: &mut Vec<Move>) {
    moves.sort_by_key(|&move_| {
        let piece = pos.board[move_.from];
        let target_piece = pos.board[move_.to];
        // score most valuable victim and least valuable attacker (MVV-LVA)
        if get_piece_type(target_piece) != EMPTY {
            return -10 * get_material_score(target_piece) + get_material_score(piece);
        }
        0
    })
}
