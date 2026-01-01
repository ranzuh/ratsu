use crate::{
    evaluation::get_material_score,
    movegen::Move,
    piece::{EMPTY, get_piece_type},
    position::Position,
    search::SearchContext,
};

pub fn order_moves_inplace(pos: &Position, moves: &mut Vec<Move>, ply: u32, info: &SearchContext) {
    // Check if we have a PV move at this ply
    let pv_move = if ply < info.prev_pv.count as u32 {
        info.prev_pv.moves[ply as usize]
    } else {
        None
    };

    moves.sort_by_key(|&move_| {
        if pv_move.is_some_and(|pv_m| pv_m == move_) {
            return -10000;
        }
        let piece = pos.board[move_.from];
        let target_piece = pos.board[move_.to];
        // score most valuable victim and least valuable attacker (MVV-LVA)
        if get_piece_type(target_piece) != EMPTY {
            return -10 * get_material_score(target_piece) + get_material_score(piece);
        }
        0
    })
}
