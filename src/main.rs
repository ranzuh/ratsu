const EMPTY: u8 = 0b00000;
const PAWN: u8 = 0b00001;
const KNIGHT: u8 = 0b00010;
const BISHOP: u8 = 0b00011;
const ROOK: u8 = 0b00100;
const QUEEN: u8 = 0b00101;
const KING: u8 = 0b00110;
const WHITE: u8 = 0b01000;
const BLACK: u8 = 0b10000;

#[derive(Clone, Copy)]
struct Position {
    board: [u8; 128],
    is_white_turn: bool,
}

fn get_piece_color(piece: u8) -> u8 {
    piece & 0b11000
}

fn get_piece_type(piece: u8) -> u8 {
    piece & 0b00111
}

const PIECES_STRING: &str = ".pnbrqkPNBRQK";

fn get_piece_char(piece: u8) -> char {
    let piece_type = get_piece_type(piece);
    let is_white = get_piece_color(piece) == WHITE;
    let n = piece_type + (is_white as u8) * 6;
    PIECES_STRING.chars().nth(n as usize).unwrap()
}

fn is_off_board(index: usize) -> bool {
    index & 0x88 != 0
}

fn print_position(position: &Position) {
    let side_to_move = match position.is_white_turn {
        true => "White",
        false => "Black",
    };
    print!("{} to move", side_to_move);

    let mut rank = 8;
    for i in 0..128 {
        if is_off_board(i) {
            continue;
        }
        if i % 16 == 0 {
            print!("\n{} ", rank);
            rank -= 1;
        }
        print!("{} ", get_piece_char(position.board[i]));
    }
    println!("\n  a b c d e f g h");
}

fn piece_from_char(char: char) -> u8 {
    let is_white = char.is_ascii_uppercase();
    let piece = match char.to_ascii_lowercase() {
        'p' => PAWN,
        'n' => KNIGHT,
        'b' => BISHOP,
        'r' => ROOK,
        'q' => QUEEN,
        'k' => KING,
        _ => EMPTY,
    };
    if is_white {
        piece | WHITE
    } else {
        piece | BLACK
    }
}

fn get_position_from_fen(fen_string: &str) -> Position {
    let mut pos = Position {
        board: [EMPTY; 128],
        is_white_turn: false,
    };
    let fen_parts = fen_string.split(" ").collect::<Vec<&str>>();
    // currently using only the piece placement, later use side, castling, ep, etc.
    let piece_placement = fen_parts[0];
    let side_to_move = fen_parts[1];

    pos.is_white_turn = side_to_move == "w";

    let mut i: usize = 0;
    for c in piece_placement.chars() {
        if c.is_numeric() {
            let n_empty_squares = c.to_digit(10).unwrap() as usize;
            i += n_empty_squares;
        }
        else if c == '/' {
            i += 8;
        }
        else {
            let piece = piece_from_char(c);
            pos.board[i] = piece;
            i += 1;
        }
    }
    pos
}

const N: i16 = -16;
const S: i16 = 16;
const E: i16 = 1;
const W: i16 = -1;

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

#[derive(Debug)]
struct Move {
    from: usize,
    to: usize,
}

fn is_square_off_board(square: usize) -> bool {
    square & 0x88 != 0
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

// get file 0..7
fn get_file(square: usize) -> usize {
    square & 7
}

// get rank 0..7
fn get_rank(square: usize) -> usize {
    square >> 4
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

fn generate_moves(position: &Position) -> Vec<Move> {
    let mut moves: Vec<Move> = Vec::new();

    for square in 0..128 {
        if is_square_off_board(square) {
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

fn debug_generate_moves(position: &Position, moves: &Vec<Move>) {
    let mut pos_copy = *position;
    for _move in moves {
        let piece = position.board[_move.from];
        pos_copy.board[_move.to] = piece;
    }
    print_position(&pos_copy);
}

const START_POSITION_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

fn main() {
    // let random_fen = "8/1b1r2kp/1q2p1p1/pp2P1P1/3P1R2/3BK2Q/PP5P/5R2 b - - 0 30";
    // let pos = get_position_from_fen(START_POSITION_FEN);
    // print_position(&pos);
    // let piece = BLACK | QUEEN;
    // println!("{:?}", get_piece_move_patterns(piece));
    // let fen_string = "8/4p3/3p1N2/2N5/8/7p/6p1/8 b - - 0 1";
    let pos = get_position_from_fen(START_POSITION_FEN);
    print_position(&pos);
    let moves = generate_moves(&pos);
    println!("{:?}", moves);
    //debug_generate_moves(&pos, &moves);
    println!("{:?}", moves.len());
}


// 0, 1, 2, 3, 4, 5, 6, 7,                   8, 9, 10, 11, 12, 13, 14, 15, 
// 16, 17, 18, 19, 20, 21, 22, 23,           24, 25, 26, 27, 28, 29, 30, 31, 
// 32, 33, 34, 35, 36, 37, 38, 39,           40, 41, 42, 43, 44, 45, 46, 47, 
// 48, 49, 50, 51, 52, 53, 54, 55,           56, 57, 58, 59, 60, 61, 62, 63, 
// 64, 65, 66, 67, 68, 69, 70, 71,           72, 73, 74, 75, 76, 77, 78, 79, 
// 80, 81, 82, 83, 84, 85, 86, 87,           88, 89, 90, 91, 92, 93, 94, 95, 
// 96, 97, 98, 99, 100, 101, 102, 103,       104, 105, 106, 107, 108, 109, 110, 111, 
// 112, 113, 114, 115, 116, 117, 118, 119,   120, 121, 122, 123, 124, 125, 126, 127,