use crate::{
    movegen::{get_file, get_rank, is_off_board},
    piece::{
        BISHOP, BLACK, EMPTY, KING, KNIGHT, PAWN, QUEEN, ROOK, WHITE, get_piece_color,
        get_piece_type,
    },
    position::Position,
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
    let file = get_file(square);
    let rank = get_rank(square);
    let square64 = rank * 8 + file;

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
    let side = match position.is_white_turn {
        true => 1,
        false => -1,
    };
    for square in 0..128 {
        if is_off_board(square) {
            continue;
        }
        let piece = position.board[square];
        let piece_type = get_piece_type(piece);
        if piece_type == EMPTY {
            continue;
        }

        score += get_piece_table_score(square, piece, piece_type);
        score += get_piece_material_score(piece);
    }
    score * side
}
