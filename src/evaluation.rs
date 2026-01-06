use crate::{
    movegen::{get_file, get_rank},
    piece::{
        BISHOP, BLACK, EMPTY, KING, KNIGHT, PAWN, QUEEN, ROOK, WHITE, get_piece_color,
        get_piece_type,
    },
    position::{Position, get_square_in_64},
};

const MATERIAL_PAWN: i32 = 100;
const MATERIAL_KNIGHT: i32 = 320;
const MATERIAL_BISHOP: i32 = 330;
const MATERIAL_ROOK: i32 = 500;
const MATERIAL_QUEEN: i32 = 900;
const MATERIAL_KING: i32 = 20000;

#[rustfmt::skip]
const PAWN_PST: [i32; 64] = [
     0,  0,  0,  0,  0,  0,  0,  0,
    50, 50, 50, 50, 50, 50, 50, 50,
    10, 10, 20, 30, 30, 20, 10, 10,
     5,  5, 10, 25, 25, 10,  5,  5,
     0,  0,  0, 20, 20,  0,  0,  0,
     5, -5,-10,  0,  0,-10, -5,  5,
     5, 10, 10,-20,-20, 10, 10,  5,
     0,  0,  0,  0,  0,  0,  0,  0
];

#[rustfmt::skip]
const KNIGHT_PST: [i32; 64] = [
    -50,-40,-30,-30,-30,-30,-40,-50,
    -40,-20,  0,  0,  0,  0,-20,-40,
    -30,  0, 10, 15, 15, 10,  0,-30,
    -30,  5, 15, 20, 20, 15,  5,-30,
    -30,  0, 15, 20, 20, 15,  0,-30,
    -30,  5, 10, 15, 15, 10,  5,-30,
    -40,-20,  0,  5,  5,  0,-20,-40,
    -50,-40,-30,-30,-30,-30,-40,-50,
];

#[rustfmt::skip]
const BISHOP_PST: [i32; 64] = [
    -20,-10,-10,-10,-10,-10,-10,-20,
    -10,  0,  0,  0,  0,  0,  0,-10,
    -10,  0,  5, 10, 10,  5,  0,-10,
    -10,  5,  5, 10, 10,  5,  5,-10,
    -10,  0, 10, 10, 10, 10,  0,-10,
    -10, 10, 10, 10, 10, 10, 10,-10,
    -10,  5,  0,  0,  0,  0,  5,-10,
    -20,-10,-10,-10,-10,-10,-10,-20,
];

#[rustfmt::skip]
const ROOK_PST: [i32; 64] = [
     0,  0,  0,  0,  0,  0,  0,  0,
     5, 10, 10, 10, 10, 10, 10,  5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
     0,  0,  0,  5,  5,  0,  0,  0,
];

#[rustfmt::skip]
const QUEEN_PST: [i32; 64] = [
    -20,-10,-10, -5, -5,-10,-10,-20,
    -10,  0,  0,  0,  0,  0,  0,-10,
    -10,  0,  5,  5,  5,  5,  0,-10,
     -5,  0,  5,  5,  5,  5,  0, -5,
      0,  0,  5,  5,  5,  5,  0, -5,
    -10,  5,  5,  5,  5,  5,  0,-10,
    -10,  0,  5,  0,  0,  0,  0,-10,
    -20,-10,-10, -5, -5,-10,-10,-20,
];

//king middle game
#[rustfmt::skip]
const KING_MG_PST: [i32; 64] = [
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -20,-30,-30,-40,-40,-30,-30,-20,
    -10,-20,-20,-20,-20,-20,-20,-10,
     20, 20,  0,  0,  0,  0, 20, 20,
     20, 30, 10,  0,  0, 10, 30, 20,
];

//king end game
#[rustfmt::skip]
const KING_EG_PST: [i32; 64] = [
    -50,-40,-30,-20,-20,-30,-40,-50,
    -30,-20,-10,  0,  0,-10,-20,-30,
    -30,-10, 20, 30, 30, 20,-10,-30,
    -30,-10, 30, 40, 40, 30,-10,-30,
    -30,-10, 30, 40, 40, 30,-10,-30,
    -30,-10, 20, 30, 30, 20,-10,-30,
    -30,-30,  0,  0,  0,  0,-30,-30,
    -50,-30,-30,-30,-30,-30,-30,-50
];

const DOUBLED_PAWN_PENALTY: i32 = 10;
const ISOLATED_PAWN_PENALTY: i32 = 20;
const BACKWARDS_PAWN_PENALTY: i32 = 8;
const PASSED_PAWN_BONUS: i32 = 20;

fn init_pawn_ranks(pos: &Position) -> ([u8; 10], [u8; 10]) {
    let mut white_pawn_ranks = [0u8; 10];
    let mut black_pawn_ranks = [7u8; 10];

    for rank in 1..7 {
        for file in 0..8 {
            let square = rank * 16 + file;
            let piece = pos.board[square];
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

const fn flip_board<T: Copy>(board: &[T; 64]) -> [T; 64] {
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
const KING_PST_MG_BLACK: [i32; 64] = flip_board(&KING_MG_PST);
const KING_PST_EG_BLACK: [i32; 64] = flip_board(&KING_EG_PST);

fn get_piece_table_score(square: usize, piece: u8, piece_type: u8) -> i32 {
    let square64 = get_square_in_64(square);

    if get_piece_color(piece) == WHITE {
        match piece_type {
            PAWN => PAWN_PST[square64],
            KNIGHT => KNIGHT_PST[square64],
            BISHOP => BISHOP_PST[square64],
            ROOK => ROOK_PST[square64],
            QUEEN => QUEEN_PST[square64],
            KING => KING_MG_PST[square64],
            _ => panic!("Unexpected piece {}", piece),
        }
    } else {
        match piece_type {
            PAWN => -PAWN_PST_BLACK[square64],
            KNIGHT => -KNIGHT_PST_BLACK[square64],
            BISHOP => -BISHOP_PST_BLACK[square64],
            ROOK => -ROOK_PST_BLACK[square64],
            QUEEN => -QUEEN_PST_BLACK[square64],
            KING => -KING_PST_MG_BLACK[square64],
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
    let (white_pawn_ranks, black_pawn_ranks) = init_pawn_ranks(position);

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
            if piece_type == PAWN {
                score += get_pawn_structure_score(
                    &white_pawn_ranks,
                    &black_pawn_ranks,
                    piece,
                    rank as u8,
                    file + 1,
                );
            }
        }
    }
    score * side
}
