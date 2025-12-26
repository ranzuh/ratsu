use crate::piece::*;
use crate::position::Position;

#[derive(Debug, Clone, Copy)]
pub struct Move {
    pub from: usize,
    pub to: usize,
    pub promoted_piece: Option<u8>,
    pub is_capture: bool,
    pub is_enpassant: bool,
    pub is_double_pawn: bool,
    pub is_castling: bool,
}

const N: isize = -16;
const S: isize = 16;
const E: isize = 1;
const W: isize = -1;

const PAWN_MOVES: &[isize] = &[N, N + N, N + W, N + E];
const KNIGHT_MOVES: &[isize] = &[
    N + N + E,
    E + E + N,
    E + E + S,
    S + S + E,
    S + S + W,
    W + W + S,
    W + W + N,
    N + N + W,
];
const BISHOP_MOVES: &[isize] = &[N + E, E + S, S + W, W + N];
const ROOK_MOVES: &[isize] = &[N, E, S, W];
const QUEEN_KING_MOVES: &[isize] = &[N, N + E, E, E + S, S, S + W, W, W + N];

fn get_piece_move_patterns(piece: u8) -> &'static [isize] {
    match get_piece_type(piece) {
        PAWN => PAWN_MOVES,
        KNIGHT => KNIGHT_MOVES,
        BISHOP => BISHOP_MOVES,
        ROOK => ROOK_MOVES,
        QUEEN | KING => QUEEN_KING_MOVES,
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

pub fn get_square_string(square: usize) -> String {
    if is_off_board(square) {
        panic!("Square {} is off board!", square);
    }
    let ranks = "87654321";
    let files = "abcdefgh";
    let rank = get_rank(square);
    let file = get_file(square);
    let rank_char = ranks.chars().nth(rank).unwrap();
    let file_char = files.chars().nth(file).unwrap();
    format!("{}{}", file_char, rank_char)
}

pub fn get_move_string(move_: &Move) -> String {
    format!(
        "{}{}",
        get_square_string(move_.from),
        get_square_string(move_.to)
    )
}

pub fn is_square_attacked(square: usize, position: &Position) -> bool {
    // pawn attacks
    if position.is_white_turn {
        let patterns = [-15, -17];
        for pattern in patterns {
            let attack = square.wrapping_add_signed(pattern);
            if is_off_board(attack) {
                continue;
            }
            let attack_piece = position.board[attack];
            if attack_piece == BLACK | PAWN {
                return true;
            }
        }
    } else {
        let patterns = [15, 17];
        for pattern in patterns {
            let attack = square.wrapping_add_signed(pattern);
            if is_off_board(attack) {
                continue;
            }
            let attack_piece = position.board[attack];
            if attack_piece == WHITE | PAWN {
                return true;
            }
        }
    }
    // knight attacks
    let patterns = get_piece_move_patterns(KNIGHT);
    for pattern in patterns {
        let attack = square.wrapping_add_signed(*pattern);
        if is_off_board(attack) {
            continue;
        }
        let attack_piece = position.board[attack];
        if position.is_white_turn && attack_piece == BLACK | KNIGHT {
            return true;
        }
        if !position.is_white_turn && attack_piece == WHITE | KNIGHT {
            return true;
        }
    }
    // king attacks
    let patterns = get_piece_move_patterns(KING);
    for pattern in patterns {
        let attack = square.wrapping_add_signed(*pattern);
        if is_off_board(attack) {
            continue;
        }
        let attack_piece = position.board[attack];
        if position.is_white_turn && attack_piece == BLACK | KING {
            return true;
        }
        if !position.is_white_turn && attack_piece == WHITE | KING {
            return true;
        }
    }
    // bishop and queen attacks
    let patterns = get_piece_move_patterns(BISHOP);
    for pattern in patterns {
        let mut attack = square.wrapping_add_signed(*pattern);
        while !is_off_board(attack) {
            let attack_piece = position.board[attack];
            if position.is_white_turn
                && (attack_piece == BLACK | BISHOP || attack_piece == BLACK | QUEEN)
            {
                return true;
            }
            if !position.is_white_turn
                && (attack_piece == WHITE | BISHOP || attack_piece == WHITE | QUEEN)
            {
                return true;
            }
            if attack_piece != EMPTY {
                break;
            }
            attack = attack.wrapping_add_signed(*pattern);
        }
    }
    // rook and queen attacks
    let patterns = get_piece_move_patterns(ROOK);
    for pattern in patterns {
        let mut attack = square.wrapping_add_signed(*pattern);
        while !is_off_board(attack) {
            let attack_piece = position.board[attack];
            if position.is_white_turn
                && (attack_piece == BLACK | ROOK || attack_piece == BLACK | QUEEN)
            {
                return true;
            }
            if !position.is_white_turn
                && (attack_piece == WHITE | ROOK || attack_piece == WHITE | QUEEN)
            {
                return true;
            }
            if attack_piece != EMPTY {
                break;
            }
            attack = attack.wrapping_add_signed(*pattern);
        }
    }

    false
}

fn generate_sliding_moves(square: usize, position: &Position, moves: &mut Vec<Move>) {
    let piece = position.board[square];
    for pattern in get_piece_move_patterns(piece) {
        let mut target_square = square;
        loop {
            target_square = target_square.wrapping_add_signed(*pattern);
            if is_off_board(target_square) {
                break;
            }
            let target_piece = position.board[target_square];
            if get_piece_color(piece) == get_piece_color(target_piece) {
                break;
            } else if target_piece != EMPTY {
                moves.push(Move {
                    from: square,
                    to: target_square,
                    promoted_piece: None,
                    is_capture: true,
                    is_enpassant: false,
                    is_double_pawn: false,
                    is_castling: false,
                });
                break;
            } else {
                moves.push(Move {
                    from: square,
                    to: target_square,
                    promoted_piece: None,
                    is_capture: false,
                    is_enpassant: false,
                    is_double_pawn: false,
                    is_castling: false,
                });
            }
        }
    }
}

fn generate_crawling_moves(square: usize, position: &Position, moves: &mut Vec<Move>) {
    let piece = position.board[square];

    if get_piece_type(piece) == KING {
        if position.is_white_turn {
            if position.castling_rights[0] {
                if position.board[square + 1] == EMPTY
                    && position.board[square + 2] == EMPTY
                    && !is_square_attacked(square, position)
                    && !is_square_attacked(square + 1, position)
                {
                    moves.push(Move {
                        from: square,
                        to: square + 2,
                        promoted_piece: None,
                        is_capture: false,
                        is_enpassant: false,
                        is_double_pawn: false,
                        is_castling: true,
                    });
                }
            }
            if position.castling_rights[1] {
                if position.board[square - 1] == EMPTY
                    && position.board[square - 2] == EMPTY
                    && position.board[square - 3] == EMPTY
                    && !is_square_attacked(square, position)
                    && !is_square_attacked(square - 1, position)
                {
                    moves.push(Move {
                        from: square,
                        to: square - 2,
                        promoted_piece: None,
                        is_capture: false,
                        is_enpassant: false,
                        is_double_pawn: false,
                        is_castling: true,
                    });
                }
            }
        }

        if !position.is_white_turn && (position.castling_rights[2] || position.castling_rights[3]) {
            if position.castling_rights[2] {
                if position.board[square + 1] == EMPTY
                    && position.board[square + 2] == EMPTY
                    && !is_square_attacked(square, position)
                    && !is_square_attacked(square + 1, position)
                {
                    moves.push(Move {
                        from: square,
                        to: square + 2,
                        promoted_piece: None,
                        is_capture: false,
                        is_enpassant: false,
                        is_double_pawn: false,
                        is_castling: true,
                    });
                }
            }
            if position.castling_rights[3] {
                if position.board[square - 1] == EMPTY
                    && position.board[square - 2] == EMPTY
                    && position.board[square - 3] == EMPTY
                    && !is_square_attacked(square, position)
                    && !is_square_attacked(square - 1, position)
                {
                    moves.push(Move {
                        from: square,
                        to: square - 2,
                        promoted_piece: None,
                        is_capture: false,
                        is_enpassant: false,
                        is_double_pawn: false,
                        is_castling: true,
                    });
                }
            }
        }
    }

    for pattern in get_piece_move_patterns(piece) {
        let target_square = square.wrapping_add_signed(*pattern);
        if is_off_board(target_square) {
            continue;
        }
        let target_piece = position.board[target_square];
        if get_piece_color(piece) == get_piece_color(target_piece) {
            continue;
        } else if target_piece != EMPTY {
            moves.push(Move {
                from: square,
                to: target_square,
                promoted_piece: None,
                is_capture: true,
                is_enpassant: false,
                is_double_pawn: false,
                is_castling: false,
            });
        } else {
            moves.push(Move {
                from: square,
                to: target_square,
                promoted_piece: None,
                is_capture: false,
                is_enpassant: false,
                is_double_pawn: false,
                is_castling: false,
            });
        }
    }
}

fn generate_pawn_moves(square: usize, position: &Position, moves: &mut Vec<Move>) {
    let is_white = position.is_white_turn;
    let piece = position.board[square];

    // Direction constants based on color
    let (forward, rank_for_double_move, promotion_rank) =
        if is_white { (N, 6, 0) } else { (S, 1, 7) };

    let current_color = if is_white { WHITE } else { BLACK };
    let opponent_color = if is_white { BLACK } else { WHITE };

    // Forward move
    let target_square = square.wrapping_add_signed(forward);
    if !is_off_board(target_square) {
        let target_piece = position.board[target_square];

        if get_piece_type(target_piece) == EMPTY {
            // Handle promotion
            if get_rank(target_square) == promotion_rank {
                for prom_piece in [KNIGHT, BISHOP, ROOK, QUEEN] {
                    moves.push(Move {
                        from: square,
                        to: target_square,
                        promoted_piece: Some(current_color | prom_piece),
                        is_capture: false,
                        is_enpassant: false,
                        is_double_pawn: false,
                        is_castling: false,
                    });
                }
            } else {
                // Normal forward move
                moves.push(Move {
                    from: square,
                    to: target_square,
                    promoted_piece: None,
                    is_capture: false,
                    is_enpassant: false,
                    is_double_pawn: false,
                    is_castling: false,
                });

                // Double forward move from starting position
                if get_rank(square) == rank_for_double_move {
                    let double_target = target_square.wrapping_add_signed(forward);
                    if get_piece_type(position.board[double_target]) == EMPTY {
                        moves.push(Move {
                            from: square,
                            to: double_target,
                            promoted_piece: None,
                            is_capture: false,
                            is_enpassant: false,
                            is_double_pawn: true,
                            is_castling: false,
                        });
                    }
                }
            }
        }
    }

    // Diagonal captures
    for diagonal in [forward + E, forward + W] {
        let target_square = square.wrapping_add_signed(diagonal);
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
                for prom_piece in [KNIGHT, BISHOP, ROOK, QUEEN] {
                    moves.push(Move {
                        from: square,
                        to: target_square,
                        promoted_piece: Some(current_color | prom_piece),
                        is_capture: false,
                        is_enpassant: false,
                        is_double_pawn: false,
                        is_castling: false,
                    });
                }
            } else {
                moves.push(Move {
                    from: square,
                    to: target_square,
                    promoted_piece: None,
                    is_capture: true,
                    is_enpassant: false,
                    is_double_pawn: false,
                    is_castling: false,
                });
            }
        }

        let ep_square = position.enpassant_square.unwrap_or(127);
        if target_square == ep_square {
            moves.push(Move {
                from: square,
                to: target_square,
                promoted_piece: None,
                is_capture: true,
                is_enpassant: true,
                is_double_pawn: false,
                is_castling: false,
            })
        }
    }
}

pub fn generate_pseudo_moves(position: &Position) -> Vec<Move> {
    let mut moves: Vec<Move> = Vec::with_capacity(100);

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
            BISHOP | ROOK | QUEEN => generate_sliding_moves(square, position, &mut moves),
            KNIGHT | KING => generate_crawling_moves(square, position, &mut moves),
            PAWN => generate_pawn_moves(square, position, &mut moves),
            _ => continue,
        }
    }

    moves
}

pub fn generate_legal_moves(position: &mut Position) -> Vec<Move> {
    let pseudo_moves = generate_pseudo_moves(position);
    let mut legal_moves: Vec<Move> = Vec::with_capacity(100);
    for move_ in &pseudo_moves {
        let piece_at_target = position.board[move_.to];
        let original_castling_rights = position.castling_rights;
        let original_king_squares = position.king_squares;
        let original_ep_square = position.enpassant_square;
        position.make_move(move_); // make move
        position.is_white_turn = !position.is_white_turn; // consider from same side before move
        let idx = match position.is_white_turn {
            true => 0,
            false => 1,
        };
        if !is_square_attacked(position.king_squares[idx], position) {
            legal_moves.push(*move_); // TODO DEBUG
        }
        position.is_white_turn = !position.is_white_turn;
        position.unmake_move(
            move_,
            piece_at_target,
            original_castling_rights,
            original_king_squares,
            original_ep_square,
        );
    }
    legal_moves
}
