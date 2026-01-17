use crate::{movegen::Move, piece::get_piece_type, position::Position};

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
    pv_move: Option<&Move>,
    tt_move: Option<&Move>,
    killers: [[Option<Move>; 2]; 64],
    history: [[u32; 128]; 128],
) {
    moves.sort_by_cached_key(|&move_| {
        if pv_move.is_some_and(|pv_m| pv_m.from == move_.from && pv_m.to == move_.to) {
            return -100;
        }
        if tt_move.is_some_and(|tt_m| tt_m.from == move_.from && tt_m.to == move_.to) {
            return -99;
        }
        // score most valuable victim and least valuable attacker (MVV-LVA)
        if move_.is_capture {
            let target_piece = pos.board[move_.to];
            let piece = pos.board[move_.from];
            let piece_type = get_piece_type(piece);
            let target_piece_type = get_piece_type(target_piece);
            return MVV_LVA[target_piece_type as usize][piece_type as usize] as i32;
        }
        if killers[ply as usize][0]
            .is_some_and(|k_mv| k_mv.from == move_.from && k_mv.to == move_.to)
        {
            return 100;
        }
        if killers[ply as usize][1]
            .is_some_and(|k_mv| k_mv.from == move_.from && k_mv.to == move_.to)
        {
            return 150;
        }
        assert!(history[move_.from][move_.to] < 1000000);
        1000150 - (history[move_.from][move_.to] as i32)
    })
}
