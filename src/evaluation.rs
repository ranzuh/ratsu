use crate::{
    bitboard::{NOT_A, NOT_H, RANK_2, RANK_7, adjacent_files, file_fill, north_fill, south_fill},
    piece::{
        BISHOP, BLACK, EMPTY, KING, KNIGHT, PAWN, QUEEN, ROOK, WHITE, get_piece_color,
        get_piece_type,
    },
    position::{Position, get_square_in_64},
};

pub const MATERIAL_PAWN: i32 = 100;
pub const MATERIAL_KNIGHT: i32 = 334;
pub const MATERIAL_BISHOP: i32 = 353;
pub const MATERIAL_ROOK: i32 = 522;
pub const MATERIAL_QUEEN: i32 = 1037;
pub const MATERIAL_KING: i32 = 20000;

#[rustfmt::skip]
pub const PAWN_MG_PST: [i32; 64] = [
      0,    0,    0,    0,    0,    0,    0,    0,
     -8,  -19,  -32,    1,   32,   11,  -44,  -84,
    -45,  -45,   -6,  -29,   -1,   60,  -32,  -45,
    -26,  -19,  -14,   -6,   13,    0,  -15,  -46,
    -47,  -27,  -24,   -6,   -5,   -9,  -23,  -64,
    -38,  -31,  -16,  -27,   -6,  -21,   13,  -27,
    -42,  -21,  -32,  -40,  -20,   15,   27,  -26,
      0,    0,    0,    0,    0,    0,    0,    0,
];

#[rustfmt::skip]
pub const KNIGHT_MG_PST: [i32; 64] = [
    -241,  -53,  -38,  -51,   51, -112, -120, -122,
     -62,  -19,   84,   40,  103,   90,   19,   20,
       4,   32,   47,   80,  114,  145,   84,   18,
      32,   33,   33,   74,   42,   62,   23,   49,
       4,   20,   35,   33,   44,   45,   32,   -7,
      -4,   14,   38,   34,   46,   32,   46,   -4,
      -3,   -8,   16,   22,   24,   36,   17,   15,
     -38,   -2,  -30,  -13,  -11,    8,   -4,  -61,
];

#[rustfmt::skip]
pub const BISHOP_MG_PST: [i32; 64] = [
    -13,  -28,    7,  -35,  -94,  -22,   17,  -37,
    -12,   33,   12,   31,   43,   86,   28,   65,
      6,   32,   78,   48,   94,   83,   75,   66,
     -2,   23,   47,   57,   56,   34,   35,   23,
     13,   36,   37,   57,   63,   38,   37,    0,
     47,   49,   44,   39,   43,   42,   39,   44,
     15,   57,   38,   30,   33,   54,   64,   36,
     34,   11,   24,   14,    2,    8,   21,   13,
];

#[rustfmt::skip]
pub const ROOK_MG_PST: [i32; 64] = [
     19,   54,   38,   69,   82,  111,   96,   36,
     -3,  -18,   27,   62,   19,   80,   95,   67,
    -12,   -8,   -7,   15,   31,   56,   67,   43,
    -41,  -27,   -5,   13,  -10,   -3,   25,    4,
    -38,  -44,  -46,  -35,  -23,  -25,   -5,  -34,
    -40,  -35,  -25,  -13,  -14,  -20,    6,  -17,
    -44,  -34,  -38,  -31,  -26,  -17,   -1,  -75,
    -17,  -21,   -8,   -9,   -5,   -8,  -53,  -28,
];

#[rustfmt::skip]
pub const QUEEN_MG_PST: [i32; 64] = [
    -72,  -17,   26,   69,   83,  120,   -1,  -48,
    -47,  -47,  -44,  -66,  -54,   76,    7,  137,
    -38,  -28,   -4,   14,   25,   76,  106,   58,
    -41,  -30,  -19,  -20,  -18,  -12,  -28,    5,
    -30,  -16,  -21,  -23,   -4,  -13,    0,  -24,
    -28,   -3,  -14,  -10,  -15,   -7,   -4,  -17,
    -27,   -8,   -4,   -6,    0,   18,   20,   -3,
     -2,  -26,  -15,    1,  -21,  -34,  -57,  -17,
];

#[rustfmt::skip]
pub const KING_MG_PST: [i32; 64] = [
    127,   71,  107,   91,  120,   99,   64,   78,
     54,  123,   77,   54,   53,  121,   22,  -40,
     64,  147,   27,   50,   11,   43,   57,   -2,
      2,    0,   46,    3,  -14,  -17,   -6,  -60,
     45,   18,   10,  -68,  -50,  -72,  -51,  -97,
     10,    6,  -74,  -95, -101,  -66,  -29,  -55,
     39,  -19,  -44,  -96,  -81,  -41,   -4,   -4,
    -58,   20,   -7,  -82,  -12,  -52,   14,   10,
];

#[rustfmt::skip]
pub const PAWN_EG_PST: [i32; 64] = [
      0,    0,    0,    0,    0,    0,    0,    0,
    186,  178,  160,  124,  108,  126,  169,  173,
    110,  100,   81,   62,   43,   37,   72,   84,
     36,   23,   13,   -2,  -10,    0,   10,   17,
     16,    3,   -5,  -11,  -11,   -9,   -6,    3,
      6,    0,   -6,    4,   -4,    1,  -10,   -6,
     20,    4,   15,   13,    4,    4,  -10,   -6,
      0,    0,    0,    0,    0,    0,    0,    0,
];

#[rustfmt::skip]
pub const KNIGHT_EG_PST: [i32; 64] = [
    -41,  -94,  -62,  -49,  -78,  -58,  -72, -130,
    -66,  -52,  -64,  -50,  -75,  -76,  -75,  -99,
    -70,  -53,  -25,  -25,  -56,  -55,  -70,  -67,
    -57,  -20,  -14,   -6,   -7,  -17,  -21,  -63,
    -43,  -37,  -10,   -9,   -4,  -26,  -45,  -34,
    -54,  -32,  -30,  -18,  -20,  -21,  -50,  -64,
    -69,  -59,  -47,  -30,  -33,  -48,  -60,  -67,
    -97,  -70,  -53,  -40,  -51,  -47,  -74,  -76,
];

#[rustfmt::skip]
pub const BISHOP_EG_PST: [i32; 64] = [
    -48,  -60,  -62,  -42,  -34,  -44,  -61,  -44,
    -56,  -50,  -42,  -54,  -45,  -63,  -49,  -85,
    -39,  -42,  -43,  -32,  -50,  -33,  -46,  -57,
    -34,  -25,  -34,  -24,  -28,  -37,  -39,  -52,
    -49,  -43,  -24,  -33,  -32,  -25,  -48,  -55,
    -58,  -42,  -32,  -25,  -21,  -31,  -55,  -60,
    -52,  -65,  -49,  -36,  -30,  -50,  -50,  -67,
    -74,  -53,  -70,  -45,  -46,  -55,  -66,  -60,
];

#[rustfmt::skip]
pub const ROOK_EG_PST: [i32; 64] = [
     15,    1,    5,  -10,  -12,  -17,  -16,    1,
     18,   29,   16,    2,    7,    2,   -3,   -5,
      5,    4,    0,   -9,  -13,  -12,  -12,  -13,
      6,    2,    0,   -9,   -5,   -1,  -10,   -8,
     -1,    3,    5,    4,    2,    1,  -11,  -12,
    -10,   -5,   -7,  -13,   -9,   -9,  -22,  -21,
     -5,   -4,    2,   -2,   -5,   -3,  -13,    0,
      0,    5,   -5,    0,  -10,    0,   10,  -15,
];

#[rustfmt::skip]
pub const QUEEN_EG_PST: [i32; 64] = [
     20,   12,    2,    2,  -16,  -27,   -6,   50,
     -1,    7,   32,   78,   68,   45,   46,  -89,
    -25,   -2,   23,   20,   61,    8,  -30,  -23,
    -18,   18,   11,   45,   64,   62,   56,   12,
     -4,    6,   17,   52,   26,   25,   19,    7,
    -29,  -39,   22,    6,   24,   11,   19,  -13,
    -13,    1,  -16,    0,   -4,  -50,  -62,  -37,
    -29,  -24,  -25,  -65,  -11,  -28,  -24,  -51,
];

#[rustfmt::skip]
pub const KING_EG_PST: [i32; 64] = [
    -94,  -47,  -54,  -43,  -44,  -30,  -30,  -62,
    -46,  -25,  -25,  -21,  -17,   -8,    6,   -1,
    -41,  -24,   -7,  -11,   -1,    8,   12,   -5,
    -45,  -16,  -10,    2,    3,   13,    6,   -5,
    -60,  -24,   -4,   15,   16,   18,    2,  -10,
    -50,  -21,    7,   20,   24,   16,    0,  -15,
    -53,  -20,    0,   15,   16,    6,  -10,  -29,
    -50,  -50,  -35,  -20,  -50,  -24,  -45,  -75,
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

pub const DOUBLED_PAWN_PENALTY: i32 = 13;
pub const ISOLATED_PAWN_PENALTY: i32 = 11;
pub const BACKWARDS_PAWN_PENALTY: i32 = 2;
pub const PASSED_PAWN_BONUS: i32 = 6;
pub const ROOK_SEMI_OPEN_FILE_BONUS: i32 = 21;
pub const ROOK_OPEN_FILE_BONUS: i32 = 28;
pub const ROOK_ON_SEVENTH_BONUS: i32 = -9;
pub const BISHOP_PAIR_BONUS: i32 = 27;

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
        assert_eq!(pawn_eval, -12);

        let rook_pos = Position::from_fen("2r1kr2/Rp1p1p2/8/8/8/8/rP1P2P1/2R1K1R1 w - - 0 1");
        let rook_eval = evaluate(&rook_pos);
        assert_eq!(rook_eval, -17);
    }

    #[test]
    fn test_eval_snapshots() {
        let cases = [
            // (fen, expected_eval)
            (START_POSITION_FEN, 0),
            // Doubled pawns
            ("4k3/2p5/2p5/8/4P3/8/4P3/4K3 w - - 0 1", -10),
            // Isolated pawns
            ("4k3/8/8/3p4/3P4/8/8/4K3 w - - 0 1", 0),
            // Passed pawns
            ("4k3/8/8/4P3/4p3/8/8/4K3 w - - 0 1", 0),
            // Rooks on open file
            ("4k2r/8/8/8/8/8/8/R3K3 w - - 0 1", 14),
            // Rooks on semi-open files
            ("4k2r/p7/8/8/8/8/7P/R3K3 w - - 0 1", -5),
            // Bishop pairs
            ("4k3/1b4b1/8/8/8/8/1B4B1/4K3 w - - 0 1", 0),
            // Complex endgame-ish position
            ("2rr1k2/7p/p6p/8/1p2pP2/7B/KB6/3R3R w - - 0 41", 300),
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
