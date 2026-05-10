use crate::{
    bitboard::{NOT_A, NOT_H, RANK_2, RANK_7, adjacent_files, file_fill, north_fill, south_fill},
    piece::{
        BISHOP, BLACK, EMPTY, KING, KNIGHT, PAWN, QUEEN, ROOK, WHITE, get_piece_color,
        get_piece_type,
    },
    position::{Position, get_square_in_64},
};

pub const MATERIAL_PAWN: i32 = 100;
pub const MATERIAL_KNIGHT: i32 = 305;
pub const MATERIAL_BISHOP: i32 = 325;
pub const MATERIAL_ROOK: i32 = 490;
pub const MATERIAL_QUEEN: i32 = 970;
pub const MATERIAL_KING: i32 = 20000;

#[rustfmt::skip]
pub const PAWN_MG_PST: [i32; 64] = [
      0,    0,    0,    0,    0,    0,    0,    0,
    70,   90,   70,  100,   75,   65,    0,  -40,
   -10,    0,   25,   35,   35,   60,   30,   -5,
   -20,   -5,  -10,   -5,   15,    5,    5,   -5,
   -30,  -15,  -15,   -5,    0,  -20,  -15,  -25,
   -30,  -20,  -20,  -20,   -5,  -25,    5,  -15,
   -35,  -20,  -25,  -45,  -20,   -5,   10,  -25,
     0,    0,    0,    0,    0,    0,    0,    0,
];

#[rustfmt::skip]
pub const KNIGHT_MG_PST: [i32; 64] = [
    -210, -135,  -55,  -35,   20,  -65,  -85, -140,
   -40,   -5,   30,   55,   40,  100,   15,   20,
   -15,   35,   55,   70,  110,  120,   60,   30,
    -5,    5,   35,   60,   40,   70,   20,   35,
   -20,   -5,   10,   15,   25,   20,   20,   -5,
   -40,  -20,   -5,    5,   15,    0,    5,  -20,
   -50,  -40,  -20,  -15,  -10,   -5,  -25,  -25,
   -95,  -45,  -55,  -40,  -35,  -25,  -40,  -65,
];

#[rustfmt::skip]
pub const BISHOP_MG_PST: [i32; 64] = [
    -50,  -45,  -20,  -70,  -65,  -50,  -10,  -70,
   -35,   20,   15,   10,   40,   35,   30,   15,
     5,   35,   40,   70,   55,   90,   65,   45,
     0,   15,   45,   55,   50,   45,   20,    5,
     0,   10,   20,   40,   35,   20,   15,   10,
    10,   15,   15,   20,   20,   15,   20,   20,
    10,   10,   25,    0,   10,   25,   25,   15,
   -10,   15,  -10,  -15,  -10,  -15,   15,    0,
];

#[rustfmt::skip]
pub const ROOK_MG_PST: [i32; 64] = [
     55,   75,  100,   95,   95,  115,   95,  100,
    30,   30,   55,   85,   70,   90,   80,  110,
     5,   35,   35,   35,   65,   65,  100,   80,
    -5,    0,    5,   10,   20,   15,   30,   35,
   -25,  -30,  -20,  -10,   -5,  -25,    0,    0,
   -35,  -25,  -20,  -20,  -15,  -20,   20,    0,
   -40,  -25,  -10,  -20,  -15,  -15,    5,  -25,
   -20,  -20,  -10,   -5,    0,  -15,    5,  -20,
];

#[rustfmt::skip]
pub const QUEEN_MG_PST: [i32; 64] = [
    -60,  -25,   15,   60,   55,   65,   55,    0,
   -20,  -25,  -10,  -10,  -10,   35,   15,   65,
    -5,   -5,    0,   15,   30,   80,   75,   80,
   -15,  -10,   -5,   -5,    0,   15,   20,   25,
    -5,  -10,   -5,    0,    0,   -5,   10,   15,
    -5,    0,   -5,    0,    0,    0,   20,   15,
    -5,   -5,    5,    5,    5,   15,   20,   35,
     0,  -20,  -10,    0,   -5,  -10,    5,   10,

];

#[rustfmt::skip]
pub const KING_MG_PST: [i32; 64] = [
    55,  -40,   10,  -55,  -25,   30,   60,  160,
   -90,   25,    5,   65,   95,  125,  155,   15,
   -90,   70,   35,   35,   75,  105,  165,   25,
   -80,   -5,   35,    5,    5,    5,  -65,  -75,
   -90,  -65,  -65,  -65,  -65,  -35,  -65, -130,
   -65,  -45,  -65,  -95,  -95,  -65,  -60,  -85,
    10,  -30,  -45,  -80,  -80,  -60,  -20,   -5,
     0,   25,    0, -100,  -45,  -75,    5,   10,
];

#[rustfmt::skip]
pub const PAWN_EG_PST: [i32; 64] = [
      0,    0,    0,    0,    0,    0,    0,    0,
   145,  140,  140,   95,   95,  100,  140,  160,
    95,  105,   75,   55,   50,   35,   80,   75,
    25,   20,   10,   -5,  -10,   -5,   15,   10,
     5,    5,  -10,  -10,  -15,   -5,    5,  -10,
    -5,    5,  -10,    0,   -5,   -5,    0,  -15,
     5,   10,   -5,   10,   10,    0,    5,  -10,
     0,    0,    0,    0,    0,    0,    0,    0,

];

#[rustfmt::skip]
pub const KNIGHT_EG_PST: [i32; 64] = [
    -45,  -20,  -15,  -15,  -25,  -35,  -25,  -80,
   -25,  -10,  -10,  -15,  -25,  -35,  -25,  -50,
   -10,   -5,    5,    5,  -10,  -20,  -20,  -35,
   -10,   10,   20,   20,   25,   10,   10,  -20,
    -5,    0,   25,   20,   25,   15,    0,  -20,
   -20,    0,   10,   15,   15,    5,  -10,  -20,
   -30,  -15,  -10,    0,   -5,  -10,  -20,  -15,
   -40,  -45,  -25,  -20,  -20,  -25,  -45,  -45,

];

#[rustfmt::skip]
pub const BISHOP_EG_PST: [i32; 64] = [
    -5,  -10,  -15,    0,   -5,  -15,  -25,  -20,
   -15,  -15,  -10,  -15,  -25,  -25,  -20,  -40,
    -5,  -15,   -5,  -20,  -15,  -15,  -20,  -15,
   -10,    5,   -5,    5,    0,    0,    0,  -15,
   -20,    0,    5,    0,    5,    5,   -5,  -30,
   -15,   -5,    0,    0,    5,    0,  -20,  -25,
   -20,  -20,  -25,   -5,   -5,  -20,  -10,  -40,
   -40,  -25,  -35,  -20,  -25,  -20,  -35,  -50,
];

#[rustfmt::skip]
pub const ROOK_EG_PST: [i32; 64] = [
     25,   25,   25,   25,   25,   10,   15,   15,
    30,   35,   40,   25,   25,   20,   15,    0,
    30,   30,   30,   30,   20,   15,   10,    5,
    30,   35,   40,   35,   20,   20,   15,   10,
    25,   30,   35,   30,   25,   25,   15,    5,
    20,   20,   20,   25,   20,   15,  -10,  -10,
    15,   15,   15,   25,   15,   15,    0,   10,
     5,   20,   25,   25,   15,   15,   10,    0,
];

#[rustfmt::skip]
pub const QUEEN_EG_PST: [i32; 64] = [
     30,   25,   35,   15,   15,   10,  -10,   15,
    -5,   25,   50,   65,   85,   40,   25,   -5,
    -5,   15,   45,   50,   55,   35,    0,  -20,
     0,   20,   35,   60,   70,   55,   35,   15,
   -10,   20,   20,   45,   45,   40,   20,    5,
   -25,   -5,   15,    5,   15,   20,  -15,  -25,
   -30,  -20,  -20,  -15,  -15,  -35,  -65,  -90,
   -40,  -35,  -35,  -30,  -30,  -40,  -55,  -70,
];

#[rustfmt::skip]
pub const KING_EG_PST: [i32; 64] = [
    -90,  -30,  -25,    5,  -10,   -5,  -10, -105,
     0,   15,   20,   10,   15,   20,   15,    5,
    15,   20,   30,   40,   40,   35,   25,   10,
     5,   25,   35,   45,   45,   45,   45,   20,
    -5,   20,   35,   45,   45,   35,   25,   15,
   -10,   10,   20,   35,   35,   25,   15,    5,
   -25,    0,   10,   20,   25,   15,    5,  -15,
   -55,  -35,  -20,   -5,  -20,   -5,  -25,  -55,
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

pub const DOUBLED_PAWN_PENALTY: i32 = 15;
pub const ISOLATED_PAWN_PENALTY: i32 = 10;
pub const BACKWARDS_PAWN_PENALTY: i32 = 5;
pub const PASSED_PAWN_BONUS: i32 = 5;
pub const ROOK_SEMI_OPEN_FILE_BONUS: i32 = 15;
pub const ROOK_OPEN_FILE_BONUS: i32 = 20;
pub const ROOK_ON_SEVENTH_BONUS: i32 = -5;
pub const BISHOP_PAIR_BONUS: i32 = 20;

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
