use crate::{
    movegen::Move,
    piece::{EMPTY, get_piece_type},
    position::Position,
    search::SearchContext,
};

#[rustfmt::skip]
pub const MVV_LVA: [[u8; 7]; 7] = [
    [0, 0, 0, 0, 0, 0, 0],          // victim; attacker;
    [0, 50, 51, 52, 53, 54, 55],    // pawn;   e p n b r q k
    [0, 40, 41, 42, 43, 44, 45],    // knight; e p n b r q k
    [0, 30, 31, 32, 33, 34, 35],    // bishop; e p n b r q k
    [0, 20, 21, 22, 23, 24, 25],    // rook;   e p n b r q k
    [0, 10, 11, 12, 13, 14, 15],    // queen;  e p n b r q k
    [0, 0, 0, 0, 0, 0, 0],          // king;   e p n b r q k
];

pub fn order_moves_inplace(
    pos: &Position,
    moves: &mut [Move],
    ply: u32,
    info: &SearchContext,
    history: &mut [[u32; 128]; 128],
    tt_move: Option<Move>,
) {
    // Check if we have a PV move at this ply
    let pv_move = if ply < info.prev_pv.count as u32 {
        info.prev_pv.moves[ply as usize]
    } else {
        None
    };

    moves.sort_by_key(|&move_| {
        if pv_move.is_some_and(|pv_m| pv_m == move_) {
            return -100;
        }
        if tt_move.is_some_and(|tt_m| tt_m == move_) {
            return -99;
        }
        let piece = pos.board[move_.from];
        let target_piece = pos.board[move_.to];
        // score most valuable victim and least valuable attacker (MVV-LVA)
        if get_piece_type(target_piece) != EMPTY {
            let piece_type = get_piece_type(piece);
            let target_piece_type = get_piece_type(target_piece);
            return MVV_LVA[target_piece_type as usize][piece_type as usize] as i32;
        }
        1000000 - (history[move_.from][move_.to] as i32)
    })
}
