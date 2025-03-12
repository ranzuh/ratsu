const EMPTY: u8 = 0b00000;
const PAWN: u8 = 0b00001;
const KNIGHT: u8 = 0b00010;
const BISHOP: u8 = 0b00011;
const ROOK: u8 = 0b00100;
const QUEEN: u8 = 0b00101;
const KING: u8 = 0b00110;
const WHITE: u8 = 0b01000;
const BLACK: u8 = 0b10000;

struct Position {
    board: [u8; 128],
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

fn print_position(position: Position) {
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
    };
    // currently using only the piece placement, later use side, castling, ep, etc.
    let piece_placement = fen_string.split(" ").collect::<Vec<&str>>()[0];
    let mut i: usize = 0;
    for c in piece_placement.chars() {
        if c.is_numeric() {
            let n_empty_squares = c.to_digit(10).unwrap() as usize;
            i += n_empty_squares - 1;
        }
        if c == '/' {
            i += 7;
        }
        let piece = piece_from_char(c);
        pos.board[i] = piece;
        i += 1;
    }
    pos
}

const START_POSITION_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

fn main() {
    // let random_fen = "8/1b1r2kp/1q2p1p1/pp2P1P1/3P1R2/3BK2Q/PP5P/5R2 b - - 0 30";
    let pos = get_position_from_fen(START_POSITION_FEN);
    print_position(pos);
}
