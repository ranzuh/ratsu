use crate::{
    movegen::{get_file, get_rank},
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
pub const PAWN_PST: [i32; 64] = [
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
pub const KNIGHT_PST: [i32; 64] = [
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
pub const BISHOP_PST: [i32; 64] = [
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
pub const ROOK_PST: [i32; 64] = [
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
pub const QUEEN_PST: [i32; 64] = [
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
pub const KING_PST: [i32; 64] = [
    -40,  -10,  -12,   -5,   -5,    1,   -5,  -25,
     -9,   15,    9,    8,   10,   25,   19,    4,
     -7,   21,   16,   16,   16,   27,   32,   10,
    -16,    5,   15,   17,   15,   20,   19,    3,
    -29,    0,   11,   14,   18,   16,    9,  -12,
    -23,   -1,    7,   12,   13,   13,    9,   -8,
    -16,   -3,    3,   -5,   -3,    7,    8,   -5,
    -36,    4,   -1,  -30,   -6,  -21,    2,  -18,
];

pub const DOUBLED_PAWN_PENALTY: i32 = 8;
pub const ISOLATED_PAWN_PENALTY: i32 = 14;
pub const BACKWARDS_PAWN_PENALTY: i32 = 6;
pub const PASSED_PAWN_BONUS: i32 = 9;
pub const ROOK_SEMI_OPEN_FILE_BONUS: i32 = -8;
pub const ROOK_OPEN_FILE_BONUS: i32 = 18;
pub const ROOK_ON_SEVENTH_BONUS: i32 = 5;
pub const BISHOP_PAIR_BONUS: i32 = 45;

pub fn init_pawn_ranks(board: &[u8; 128]) -> ([u8; 10], [u8; 10]) {
    let mut white_pawn_ranks = [0u8; 10];
    let mut black_pawn_ranks = [7u8; 10];

    for rank in 1..7 {
        for file in 0..8 {
            let square = rank * 16 + file;
            let piece = board[square];
            if get_piece_type(piece) == PAWN {
                let rank = get_rank(square) as u8;
                let pawn_file_index = get_file(square) + 1;
                let is_white = get_piece_color(piece) == WHITE;
                if is_white && white_pawn_ranks[pawn_file_index] < rank {
                    white_pawn_ranks[pawn_file_index] = rank
                } else if !is_white && black_pawn_ranks[pawn_file_index] > rank {
                    black_pawn_ranks[pawn_file_index] = rank
                }
            }
        }
    }
    (white_pawn_ranks, black_pawn_ranks)
}

fn get_pawn_structure_score(
    white_pawn_ranks: &[u8; 10],
    black_pawn_ranks: &[u8; 10],
    piece: u8,
    rank: u8,
    pawn_file: usize,
) -> i32 {
    let mut score = 0;
    let left_file = pawn_file - 1;
    let right_file = pawn_file + 1;
    if get_piece_color(piece) == WHITE {
        if white_pawn_ranks[pawn_file] > rank {
            score -= DOUBLED_PAWN_PENALTY;
        }

        if white_pawn_ranks[left_file] == 0 && white_pawn_ranks[right_file] == 0 {
            score -= ISOLATED_PAWN_PENALTY;
        } else if rank > white_pawn_ranks[left_file] && rank > white_pawn_ranks[right_file] {
            score -= BACKWARDS_PAWN_PENALTY;
        }

        if rank <= black_pawn_ranks[left_file]
            && rank <= black_pawn_ranks[pawn_file]
            && rank <= black_pawn_ranks[right_file]
        {
            score += (7 - rank as i32) * PASSED_PAWN_BONUS;
        }
    } else {
        if black_pawn_ranks[pawn_file] < rank {
            score += DOUBLED_PAWN_PENALTY;
        }
        if black_pawn_ranks[left_file] == 7 && black_pawn_ranks[right_file] == 7 {
            score += ISOLATED_PAWN_PENALTY;
        } else if rank < black_pawn_ranks[left_file] && rank < black_pawn_ranks[right_file] {
            score += BACKWARDS_PAWN_PENALTY;
        }

        if rank >= white_pawn_ranks[left_file]
            && rank >= white_pawn_ranks[pawn_file]
            && rank >= white_pawn_ranks[right_file]
        {
            score -= rank as i32 * PASSED_PAWN_BONUS
        }
    }
    score
}

fn get_rook_score(
    white_pawn_ranks: &[u8; 10],
    black_pawn_ranks: &[u8; 10],
    piece: u8,
    rank: u8,
    pawn_file: usize,
) -> i32 {
    let mut score = 0;
    if get_piece_color(piece) == WHITE {
        if black_pawn_ranks[pawn_file] == 7 {
            if white_pawn_ranks[pawn_file] == 0 {
                score += ROOK_OPEN_FILE_BONUS
            } else {
                score += ROOK_SEMI_OPEN_FILE_BONUS
            }
        }

        if rank == 1 {
            score += ROOK_ON_SEVENTH_BONUS
        }
    } else {
        if white_pawn_ranks[pawn_file] == 0 {
            if black_pawn_ranks[pawn_file] == 7 {
                score -= ROOK_OPEN_FILE_BONUS
            } else {
                score -= ROOK_SEMI_OPEN_FILE_BONUS
            }
        }

        if rank == 6 {
            score -= ROOK_ON_SEVENTH_BONUS
        }
    }
    score
}

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

const PAWN_PST_BLACK: [i32; 64] = flip_board(&PAWN_PST);
const KNIGHT_PST_BLACK: [i32; 64] = flip_board(&KNIGHT_PST);
const ROOK_PST_BLACK: [i32; 64] = flip_board(&ROOK_PST);
const BISHOP_PST_BLACK: [i32; 64] = flip_board(&BISHOP_PST);
const QUEEN_PST_BLACK: [i32; 64] = flip_board(&QUEEN_PST);
const KING_PST_BLACK: [i32; 64] = flip_board(&KING_PST);

fn get_piece_table_score(square: usize, piece: u8, piece_type: u8) -> i32 {
    let square64 = get_square_in_64(square);

    if get_piece_color(piece) == WHITE {
        match piece_type {
            PAWN => PAWN_PST[square64],
            KNIGHT => KNIGHT_PST[square64],
            BISHOP => BISHOP_PST[square64],
            ROOK => ROOK_PST[square64],
            QUEEN => QUEEN_PST[square64],
            KING => KING_PST[square64],
            _ => panic!("Unexpected piece {}", piece),
        }
    } else {
        match piece_type {
            PAWN => -PAWN_PST_BLACK[square64],
            KNIGHT => -KNIGHT_PST_BLACK[square64],
            BISHOP => -BISHOP_PST_BLACK[square64],
            ROOK => -ROOK_PST_BLACK[square64],
            QUEEN => -QUEEN_PST_BLACK[square64],
            KING => -KING_PST_BLACK[square64],
            _ => panic!("Unexpected piece {}", piece),
        }
    }
}

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
    let side = if position.is_white_turn { 1 } else { -1 };
    let (white_pawn_ranks, black_pawn_ranks) = init_pawn_ranks(&position.board);
    let mut white_bishops = 0;
    let mut black_bishops = 0;

    for rank in 0..8 {
        for file in 0..8 {
            let square = rank * 16 + file;
            let piece = position.board[square];
            let piece_type = get_piece_type(piece);
            if piece_type == EMPTY {
                continue;
            }

            score += get_piece_table_score(square, piece, piece_type);
            score += get_piece_material_score(piece);
            if piece_type == BISHOP {
                if get_piece_color(piece) == WHITE {
                    white_bishops += 1;
                } else {
                    black_bishops += 1;
                }
            }
            if piece_type == PAWN {
                score += get_pawn_structure_score(
                    &white_pawn_ranks,
                    &black_pawn_ranks,
                    piece,
                    rank as u8,
                    file + 1,
                );
            }
            if piece_type == ROOK {
                score += get_rook_score(
                    &white_pawn_ranks,
                    &black_pawn_ranks,
                    piece,
                    rank as u8,
                    file + 1,
                );
            }
        }
    }
    if white_bishops >= 2 {
        score += BISHOP_PAIR_BONUS;
    }
    if black_bishops >= 2 {
        score -= BISHOP_PAIR_BONUS;
    }
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
            ("2rr1k2/7p/p6p/8/1p2pP2/7B/KB6/3R3R w - - 0 41", 278),
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
