use crate::{
    bitboard::{NOT_A, NOT_H, RANK_2, RANK_7, adjacent_files, file_fill, north_fill, south_fill},
    piece::{
        BISHOP, BLACK, EMPTY, KING, KNIGHT, PAWN, QUEEN, ROOK, WHITE, get_piece_color,
        get_piece_type,
    },
    position::{Position, get_square_in_64},
};

pub const MATERIAL_PAWN: i32 = 100;
pub const MATERIAL_KNIGHT: i32 = 350;
pub const MATERIAL_BISHOP: i32 = 350;
pub const MATERIAL_ROOK: i32 = 525;
pub const MATERIAL_QUEEN: i32 = 1000;
pub const MATERIAL_KING: i32 = 20000;

#[rustfmt::skip]
pub const PAWN_MG_PST: [i32; 64] = [
      0,    0,    0,    0,    0,    0,    0,    0,
    124,   93,   88,   68,   66,   70,   92,  102,
     48,   44,   34,    9,   17,   35,   40,   39,
      4,   -2,   -9,  -10,    0,   -6,   -2,   -4,
    -19,  -15,  -18,   -6,   -9,  -16,  -17,  -30,
    -16,  -19,  -13,  -19,   -8,  -15,   -3,  -15,
    -17,  -17,  -22,  -28,  -20,    5,    1,  -11,
      0,    0,    0,    0,    0,    0,    0,    0,
];

#[rustfmt::skip]
pub const KNIGHT_MG_PST: [i32; 64] = [
    -152,  -84,  -53,  -50,  -45,  -86,  -80, -131,
     -72,  -40,    9,  -17,   -5,   -1,  -38,  -48,
     -40,  -19,    6,   19,   13,   20,   -5,  -37,
     -19,   -2,    0,   30,   13,   16,   -7,  -15,
     -27,  -16,    6,    4,   12,    5,  -13,  -30,
     -34,  -18,    1,    1,    5,    1,   -1,  -37,
     -39,  -44,  -23,  -10,   -9,  -12,  -25,  -23,
     -75,  -37,  -52,  -34,  -38,  -23,  -39,  -70,
];

#[rustfmt::skip]
pub const BISHOP_MG_PST: [i32; 64] = [
    -32,  -39,  -27,  -25,  -27,  -29,  -25,  -36,
    -35,   -8,  -16,  -16,   -4,    4,   -8,  -11,
    -15,   -3,   13,    8,   10,   16,    7,    7,
    -16,   -8,    5,   13,   13,   -2,   -5,  -15,
    -19,   -4,    4,   11,   17,    1,   -4,  -31,
     -4,    5,    5,    6,    7,    8,   -2,   -5,
    -20,    4,   -4,   -7,   -1,    3,   19,   -9,
    -24,  -24,  -19,  -16,  -21,  -27,  -29,  -23,
];

#[rustfmt::skip]
pub const ROOK_MG_PST: [i32; 64] = [
    33,   27,   27,   23,   25,   24,   20,   23,
    20,   21,   23,   23,   15,   24,   21,   16,
    20,   19,   15,   18,   15,   21,   18,   14,
    14,   12,   18,   14,   11,   16,   14,   16,
     5,    7,    6,    9,    7,    8,    6,   -4,
    -6,    2,    3,    5,    6,    5,    2,   -5,
    -9,    1,    3,    4,    3,    8,    8,  -22,
     1,    7,   14,   17,   16,   13,  -15,  -19,
];

#[rustfmt::skip]
pub const QUEEN_MG_PST: [i32; 64] = [
     1,   23,   38,   50,   46,   50,   20,   33,
    -6,   -9,   13,   31,   28,   87,   59,   64,
    -5,   11,   35,   44,   68,   68,   72,   60,
    -5,   14,   20,   26,   42,   48,   29,   41,
     4,   20,   19,   26,   36,   29,   37,   19,
    -1,   19,   25,   25,   26,   31,   33,   15,
     6,   24,   25,   27,   31,   26,   21,   12,
    25,    6,   12,   25,   12,    5,  -18,   -1,
];

#[rustfmt::skip]
pub const KING_MG_PST: [i32; 64] = [
    -40,  -10,  -12,   -5,   -5,    1,   -5,  -25,
     -9,   15,    9,    8,   10,   25,   19,    4,
     -7,   21,   16,   16,   16,   27,   32,   10,
    -16,    5,   15,   17,   15,   20,   19,    3,
    -29,    0,   11,   14,   18,   16,    9,  -12,
    -23,   -1,    7,   12,   13,   13,    9,   -8,
    -16,   -3,    3,   -5,   -3,    7,    8,   -5,
    -36,    4,   -1,  -30,   -6,  -21,    2,  -18,
];

#[rustfmt::skip]
pub const PAWN_EG_PST: [i32; 64] = [
      0,    0,    0,    0,    0,    0,    0,    0,
    124,   93,   88,   68,   66,   70,   92,  102,
     48,   44,   34,    9,   17,   35,   40,   39,
      4,   -2,   -9,  -10,    0,   -6,   -2,   -4,
    -19,  -15,  -18,   -6,   -9,  -16,  -17,  -30,
    -16,  -19,  -13,  -19,   -8,  -15,   -3,  -15,
    -17,  -17,  -22,  -28,  -20,    5,    1,  -11,
      0,    0,    0,    0,    0,    0,    0,    0,
];

#[rustfmt::skip]
pub const KNIGHT_EG_PST: [i32; 64] = [
    -152,  -84,  -53,  -50,  -45,  -86,  -80, -131,
     -72,  -40,    9,  -17,   -5,   -1,  -38,  -48,
     -40,  -19,    6,   19,   13,   20,   -5,  -37,
     -19,   -2,    0,   30,   13,   16,   -7,  -15,
     -27,  -16,    6,    4,   12,    5,  -13,  -30,
     -34,  -18,    1,    1,    5,    1,   -1,  -37,
     -39,  -44,  -23,  -10,   -9,  -12,  -25,  -23,
     -75,  -37,  -52,  -34,  -38,  -23,  -39,  -70,
];

#[rustfmt::skip]
pub const BISHOP_EG_PST: [i32; 64] = [
    -32,  -39,  -27,  -25,  -27,  -29,  -25,  -36,
    -35,   -8,  -16,  -16,   -4,    4,   -8,  -11,
    -15,   -3,   13,    8,   10,   16,    7,    7,
    -16,   -8,    5,   13,   13,   -2,   -5,  -15,
    -19,   -4,    4,   11,   17,    1,   -4,  -31,
     -4,    5,    5,    6,    7,    8,   -2,   -5,
    -20,    4,   -4,   -7,   -1,    3,   19,   -9,
    -24,  -24,  -19,  -16,  -21,  -27,  -29,  -23,
];

#[rustfmt::skip]
pub const ROOK_EG_PST: [i32; 64] = [
    33,   27,   27,   23,   25,   24,   20,   23,
    20,   21,   23,   23,   15,   24,   21,   16,
    20,   19,   15,   18,   15,   21,   18,   14,
    14,   12,   18,   14,   11,   16,   14,   16,
     5,    7,    6,    9,    7,    8,    6,   -4,
    -6,    2,    3,    5,    6,    5,    2,   -5,
    -9,    1,    3,    4,    3,    8,    8,  -22,
     1,    7,   14,   17,   16,   13,  -15,  -19,
];

#[rustfmt::skip]
pub const QUEEN_EG_PST: [i32; 64] = [
     1,   23,   38,   50,   46,   50,   20,   33,
    -6,   -9,   13,   31,   28,   87,   59,   64,
    -5,   11,   35,   44,   68,   68,   72,   60,
    -5,   14,   20,   26,   42,   48,   29,   41,
     4,   20,   19,   26,   36,   29,   37,   19,
    -1,   19,   25,   25,   26,   31,   33,   15,
     6,   24,   25,   27,   31,   26,   21,   12,
    25,    6,   12,   25,   12,    5,  -18,   -1,
];

#[rustfmt::skip]
pub const KING_EG_PST: [i32; 64] = [
    -40,  -10,  -12,   -5,   -5,    1,   -5,  -25,
     -9,   15,    9,    8,   10,   25,   19,    4,
     -7,   21,   16,   16,   16,   27,   32,   10,
    -16,    5,   15,   17,   15,   20,   19,    3,
    -29,    0,   11,   14,   18,   16,    9,  -12,
    -23,   -1,    7,   12,   13,   13,    9,   -8,
    -16,   -3,    3,   -5,   -3,    7,    8,   -5,
    -36,    4,   -1,  -30,   -6,  -21,    2,  -18,
];

pub const fn flip_board<T: Copy>(board: &[T; 64]) -> [T; 64] {
    let mut flipped = *board;
    let mut rank = 0;
    while rank < 4 {
        let mut file = 0;
        while file < 8 {
            let top_idx = rank * 8 + file;
            let bottom_idx = (7 - rank) * 8 + file;
            // Manual swap since .swap() isn't const-stable
            let temp = flipped[top_idx];
            flipped[top_idx] = flipped[bottom_idx];
            flipped[bottom_idx] = temp;
            file += 1;
        }
        rank += 1;
    }
    flipped
}

const PAWN_MG_PST_BLACK: [i32; 64] = flip_board(&PAWN_MG_PST);
const KNIGHT_MG_PST_BLACK: [i32; 64] = flip_board(&KNIGHT_MG_PST);
const ROOK_MG_PST_BLACK: [i32; 64] = flip_board(&ROOK_MG_PST);
const BISHOP_MG_PST_BLACK: [i32; 64] = flip_board(&BISHOP_MG_PST);
const QUEEN_MG_PST_BLACK: [i32; 64] = flip_board(&QUEEN_MG_PST);
const KING_MG_PST_BLACK: [i32; 64] = flip_board(&KING_MG_PST);

const PAWN_EG_PST_BLACK: [i32; 64] = flip_board(&PAWN_EG_PST);
const KNIGHT_EG_PST_BLACK: [i32; 64] = flip_board(&KNIGHT_EG_PST);
const ROOK_EG_PST_BLACK: [i32; 64] = flip_board(&ROOK_EG_PST);
const BISHOP_EG_PST_BLACK: [i32; 64] = flip_board(&BISHOP_EG_PST);
const QUEEN_EG_PST_BLACK: [i32; 64] = flip_board(&QUEEN_EG_PST);
const KING_EG_PST_BLACK: [i32; 64] = flip_board(&KING_EG_PST);

pub const DOUBLED_PAWN_PENALTY: i32 = 8;
pub const ISOLATED_PAWN_PENALTY: i32 = 14;
pub const BACKWARDS_PAWN_PENALTY: i32 = 6;
pub const PASSED_PAWN_BONUS: i32 = 9;
pub const ROOK_SEMI_OPEN_FILE_BONUS: i32 = 8;
pub const ROOK_OPEN_FILE_BONUS: i32 = 18;
pub const ROOK_ON_SEVENTH_BONUS: i32 = 5;
pub const BISHOP_PAIR_BONUS: i32 = 45;

pub fn get_mg_piece_table_score(square: usize, piece: u8, piece_type: u8) -> i32 {
    let square64 = get_square_in_64(square);

    if get_piece_color(piece) == WHITE {
        match piece_type {
            PAWN => PAWN_MG_PST[square64],
            KNIGHT => KNIGHT_MG_PST[square64],
            BISHOP => BISHOP_MG_PST[square64],
            ROOK => ROOK_MG_PST[square64],
            QUEEN => QUEEN_MG_PST[square64],
            KING => KING_MG_PST[square64],
            _ => panic!("Unexpected piece {}", piece),
        }
    } else {
        match piece_type {
            PAWN => -PAWN_MG_PST_BLACK[square64],
            KNIGHT => -KNIGHT_MG_PST_BLACK[square64],
            BISHOP => -BISHOP_MG_PST_BLACK[square64],
            ROOK => -ROOK_MG_PST_BLACK[square64],
            QUEEN => -QUEEN_MG_PST_BLACK[square64],
            KING => -KING_MG_PST_BLACK[square64],
            _ => panic!("Unexpected piece {}", piece),
        }
    }
}

pub fn get_eg_piece_table_score(square: usize, piece: u8, piece_type: u8) -> i32 {
    let square64 = get_square_in_64(square);

    if get_piece_color(piece) == WHITE {
        match piece_type {
            PAWN => PAWN_EG_PST[square64],
            KNIGHT => KNIGHT_EG_PST[square64],
            BISHOP => BISHOP_EG_PST[square64],
            ROOK => ROOK_EG_PST[square64],
            QUEEN => QUEEN_EG_PST[square64],
            KING => KING_EG_PST[square64],
            _ => panic!("Unexpected piece {}", piece),
        }
    } else {
        match piece_type {
            PAWN => -PAWN_EG_PST_BLACK[square64],
            KNIGHT => -KNIGHT_EG_PST_BLACK[square64],
            BISHOP => -BISHOP_EG_PST_BLACK[square64],
            ROOK => -ROOK_EG_PST_BLACK[square64],
            QUEEN => -QUEEN_EG_PST_BLACK[square64],
            KING => -KING_EG_PST_BLACK[square64],
            _ => panic!("Unexpected piece {}", piece),
        }
    }
}

fn get_material_score(piece: u8) -> i32 {
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

pub fn get_piece_material_score(piece: u8) -> i32 {
    let side = match get_piece_color(piece) {
        WHITE => 1,
        BLACK => -1,
        EMPTY => 0,
        _ => panic!("{}", get_piece_color(piece)),
    };
    let material_score = get_material_score(piece);
    side * material_score
}

fn bb_pawn_structure(bb_color: &[u64; 2], bb_piece: &[u64; 6]) -> i32 {
    let white_pawns = bb_color[0] & bb_piece[0];
    let black_pawns = bb_color[1] & bb_piece[0];

    let mut score = 0;

    // double pawns
    let white_doubled = white_pawns & north_fill(white_pawns << 8);
    let black_doubled = black_pawns & south_fill(black_pawns >> 8);

    score -= DOUBLED_PAWN_PENALTY * white_doubled.count_ones() as i32;
    score += DOUBLED_PAWN_PENALTY * black_doubled.count_ones() as i32;

    // isolated pawns
    let white_isolated = white_pawns & !adjacent_files(file_fill(white_pawns));
    let black_isolated = black_pawns & !adjacent_files(file_fill(black_pawns));

    score -= ISOLATED_PAWN_PENALTY * white_isolated.count_ones() as i32;
    score += ISOLATED_PAWN_PENALTY * black_isolated.count_ones() as i32;

    // passed pawns
    let b_south = south_fill(black_pawns);
    let b_sentinel = b_south | adjacent_files(south_fill(black_pawns >> 8));
    let mut white_passed = white_pawns & !b_sentinel;
    while white_passed != 0 {
        let sq = white_passed.trailing_zeros() as i32;
        let rank = sq / 8; // 0=rank1, 7=rank8
        score += rank * PASSED_PAWN_BONUS;
        white_passed &= white_passed - 1;
    }

    let w_north = north_fill(white_pawns);
    let w_sentinel = w_north | adjacent_files(north_fill(white_pawns << 8));
    let mut black_passed = black_pawns & !w_sentinel;
    while black_passed != 0 {
        let sq = black_passed.trailing_zeros() as i32;
        let rank = sq / 8;
        score -= (7 - rank) * PASSED_PAWN_BONUS;
        black_passed &= black_passed - 1;
    }

    // backwards pawns
    let white_stop = white_pawns << 8; // square in front of each pawn
    let black_attacks = ((black_pawns & NOT_A) >> 9) | ((black_pawns & NOT_H) >> 7);
    let white_behind = white_pawns & !south_fill(adjacent_files(white_pawns));
    let white_backward = white_behind & !white_isolated & (white_stop & black_attacks) >> 8;

    let black_stop = black_pawns >> 8; // square in front of each pawn
    let white_attacks = ((white_pawns & NOT_A) << 7) | ((white_pawns & NOT_H) << 9);
    let black_behind = black_pawns & !north_fill(adjacent_files(black_pawns));
    let black_backward = black_behind & !black_isolated & (black_stop & white_attacks) << 8;

    score -= BACKWARDS_PAWN_PENALTY * white_backward.count_ones() as i32;
    score += BACKWARDS_PAWN_PENALTY * black_backward.count_ones() as i32;

    score
}

pub fn bb_rook_score(bb_color: &[u64; 2], bb_piece: &[u64; 6]) -> i32 {
    let white_pawns = bb_color[0] & bb_piece[0];
    let black_pawns = bb_color[1] & bb_piece[0];
    let white_rooks = bb_color[0] & bb_piece[3];
    let black_rooks = bb_color[1] & bb_piece[3];

    let w_pawn_files = file_fill(white_pawns);
    let b_pawn_files = file_fill(black_pawns);

    let open_files = !w_pawn_files & !b_pawn_files;
    let w_semi_open = !w_pawn_files & b_pawn_files;
    let b_semi_open = !b_pawn_files & w_pawn_files;

    let mut score = 0;

    // White rooks
    score += ROOK_OPEN_FILE_BONUS * (white_rooks & open_files).count_ones() as i32;
    score += ROOK_SEMI_OPEN_FILE_BONUS * (white_rooks & w_semi_open).count_ones() as i32;
    score += ROOK_ON_SEVENTH_BONUS * (white_rooks & RANK_7).count_ones() as i32;

    // Black rooks
    score -= ROOK_OPEN_FILE_BONUS * (black_rooks & open_files).count_ones() as i32;
    score -= ROOK_SEMI_OPEN_FILE_BONUS * (black_rooks & b_semi_open).count_ones() as i32;
    score -= ROOK_ON_SEVENTH_BONUS * (black_rooks & RANK_2).count_ones() as i32;

    score
}

pub fn evaluate(position: &Position) -> i32 {
    let mut score = 0;
    let side = if position.is_white_turn { 1 } else { -1 };

    let white_bishops = (position.bb_color[0] & position.bb_piece[2]).count_ones();
    let black_bishops = (position.bb_color[1] & position.bb_piece[2]).count_ones();
    if white_bishops >= 2 {
        score += BISHOP_PAIR_BONUS;
    }
    if black_bishops >= 2 {
        score -= BISHOP_PAIR_BONUS;
    }

    let mg_phase = position.phase_value.clamp(0, 6400);
    let eg_phase = 6400 - mg_phase;

    let tapered_pst_score =
        (position.mg_pst_score * mg_phase + position.eg_pst_score * eg_phase) / 6400;

    score += position.material_score + tapered_pst_score;
    score += bb_pawn_structure(&position.bb_color, &position.bb_piece);
    score += bb_rook_score(&position.bb_color, &position.bb_piece);

    score * side
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::START_POSITION_FEN;

    #[test]
    fn test_evaluate() {
        let pos = Position::from_fen(START_POSITION_FEN);
        let eval = evaluate(&pos);
        assert_eq!(eval, 0);

        let pawn_pos = Position::from_fen("4k3/1p2p3/4p1P1/3p4/3P4/4P2p/1P2P3/4K3 w - - 0 1");
        let pawn_eval = evaluate(&pawn_pos);
        assert_eq!(pawn_eval, 1);

        let rook_pos = Position::from_fen("2r1kr2/Rp1p1p2/8/8/8/8/rP1P2P1/2R1K1R1 w - - 0 1");
        let rook_eval = evaluate(&rook_pos);
        assert_eq!(rook_eval, -32);
    }

    #[test]
    fn test_eval_snapshots() {
        let cases = [
            // (fen, expected_eval)
            (START_POSITION_FEN, 0),
            // Doubled pawns
            ("4k3/2p5/2p5/8/4P3/8/4P3/4K3 w - - 0 1", 15),
            // Isolated pawns
            ("4k3/8/8/3p4/3P4/8/8/4K3 w - - 0 1", 0),
            // Passed pawns
            ("4k3/8/8/4P3/4p3/8/8/4K3 w - - 0 1", 0),
            // Rooks on open file
            ("4k2r/8/8/8/8/8/8/R3K3 w - - 0 1", 20),
            // Rooks on semi-open files
            ("4k2r/p7/8/8/8/8/7P/R3K3 w - - 0 1", 26),
            // Bishop pairs
            ("4k3/1b4b1/8/8/8/8/1B4B1/4K3 w - - 0 1", 0),
            // Complex endgame-ish position
            ("2rr1k2/7p/p6p/8/1p2pP2/7B/KB6/3R3R w - - 0 41", 280),
            // Rooks on 7th ranks
            ("4k3/R7/8/8/8/8/r7/4K3 w - - 0 1", 0),
        ];

        for (fen, expected) in &cases {
            let pos = Position::from_fen(fen);
            let eval = evaluate(&pos);
            assert_eq!(eval, *expected, "Failed for FEN: {}", fen);
        }
    }
}
