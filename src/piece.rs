pub const EMPTY: u8 = 0b00000;
pub const PAWN: u8 = 0b00001;
pub const KNIGHT: u8 = 0b00010;
pub const BISHOP: u8 = 0b00011;
pub const ROOK: u8 = 0b00100;
pub const QUEEN: u8 = 0b00101;
pub const KING: u8 = 0b00110;
pub const WHITE: u8 = 0b01000;
pub const BLACK: u8 = 0b10000;

pub fn get_piece_color(piece: u8) -> u8 {
    piece & 0b11000
}

pub fn get_piece_type(piece: u8) -> u8 {
    piece & 0b00111
}

const PIECES_STRING: &str = ".pnbrqkPNBRQK";

pub fn get_piece_char(piece: u8) -> char {
    let piece_type = get_piece_type(piece);
    let is_white = get_piece_color(piece) == WHITE;
    let n = piece_type + (is_white as u8) * 6;
    PIECES_STRING.chars().nth(n as usize).unwrap()
}

pub fn piece_from_char(char: char) -> u8 {
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
