use crate::piece::*;
use crate::position::Position;

const N: i16 = -16;
const S: i16 = 16;
const E: i16 = 1;
const W: i16 = -1;

#[derive(Debug)]
pub struct Move {
    pub from: usize,
    pub to: usize,
}

fn get_piece_move_patterns(piece: u8) -> &'static [i16] {
    match get_piece_type(piece) {
        PAWN => &[N, N + N, N + W, N + E],
        KNIGHT => &[
            N + N + E,
            E + E + N,
            E + E + S,
            S + S + E,
            S + S + W,
            W + W + S,
            W + W + N,
            N + N + W,
        ],
        BISHOP => &[N + E, E + S, S + W, W + N],
        ROOK => &[N, E, S, W],
        QUEEN | KING => &[N, N + E, E, E + S, S, S + W, W, W + N],
        _ => &[],
    }
}

pub fn is_off_board(index: usize) -> bool {
    index & 0x88 != 0
}

#[allow(dead_code)]
// get file 0..7
fn get_file(square: usize) -> usize {
    square & 7
}

// get rank 0..7
fn get_rank(square: usize) -> usize {
    square >> 4
}

fn generate_sliding_moves(square: usize, position: &Position, moves: &mut Vec<Move>) {
    let piece = position.board[square];
    for pattern in get_piece_move_patterns(piece) {
        let mut target_square = square;
        loop {
            target_square = ((target_square as i16) + pattern) as usize;
            if is_off_board(target_square) {
                break;
            }
            let target_piece = position.board[target_square];
            if get_piece_color(piece) == get_piece_color(target_piece) {
                break;
            }
            moves.push(Move {
                from: square,
                to: target_square,
            });
        }
    }
}

fn generate_crawling_moves(square: usize, position: &Position, moves: &mut Vec<Move>) {
    let piece = position.board[square];
    for pattern in get_piece_move_patterns(piece) {
        let target_square = ((square as i16) + pattern) as usize;
        if is_off_board(target_square) {
            continue;
        }
        let target_piece = position.board[target_square];
        if get_piece_color(piece) == get_piece_color(target_piece) {
            continue;
        }
        moves.push(Move {
            from: square,
            to: target_square,
        });
    }
}

fn generate_pawn_moves(square: usize, position: &Position, moves: &mut Vec<Move>) {
    let is_white = position.is_white_turn;
    let piece = position.board[square];

    // Direction constants based on color
    let (forward, rank_for_double_move, promotion_rank) =
        if is_white { (N, 6, 0) } else { (S, 1, 7) };

    let opponent_color = if is_white { BLACK } else { WHITE };

    // Forward move
    let target_square = ((square as i16) + forward) as usize;
    if !is_off_board(target_square) {
        let target_piece = position.board[target_square];

        if get_piece_type(target_piece) == EMPTY {
            // Handle promotion
            if get_rank(target_square) == promotion_rank {
                // TODO: handle promotion
            } else {
                // Normal forward move
                moves.push(Move {
                    from: square,
                    to: target_square,
                });

                // Double forward move from starting position
                if get_rank(square) == rank_for_double_move {
                    let double_target = ((target_square as i16) + forward) as usize;
                    if get_piece_type(position.board[double_target]) == EMPTY {
                        moves.push(Move {
                            from: square,
                            to: double_target,
                        });
                    }
                }
            }
        }
    }

    // Diagonal captures
    for diagonal in [forward + E, forward + W] {
        let target_square = ((square as i16) + diagonal) as usize;
        if is_off_board(target_square) {
            continue;
        }

        let target_piece = position.board[target_square];

        // Skip if same color piece
        if get_piece_color(piece) == get_piece_color(target_piece) {
            continue;
        }

        // Capture opponent's piece
        if get_piece_color(target_piece) == opponent_color {
            // Handle promotion
            if get_rank(target_square) == promotion_rank {
                // TODO: handle promotion
            } else {
                moves.push(Move {
                    from: square,
                    to: target_square,
                });
            }
        }
    }
}

pub fn generate_moves(position: &Position) -> Vec<Move> {
    let mut moves: Vec<Move> = Vec::new();

    for square in 0..128 {
        if is_off_board(square) {
            continue;
        }
        let piece = position.board[square];
        let side_to_move = if position.is_white_turn { WHITE } else { BLACK };
        if get_piece_color(piece) != side_to_move {
            continue;
        }
        match get_piece_type(piece) {
            BISHOP | ROOK | QUEEN => generate_sliding_moves(square, &position, &mut moves),
            KNIGHT | KING => generate_crawling_moves(square, &position, &mut moves),
            PAWN => generate_pawn_moves(square, &position, &mut moves),
            _ => continue,
        }
    }

    moves
}

#[allow(dead_code)]
fn debug_generate_moves(position: &Position, moves: &Vec<Move>) {
    let mut pos_copy = *position;
    for _move in moves {
        let piece = position.board[_move.from];
        pos_copy.board[_move.to] = piece;
    }
    pos_copy.print();
}
