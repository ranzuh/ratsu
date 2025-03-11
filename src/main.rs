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

fn initialize_position(position: &mut Position) {
    position.board[0] = BLACK | ROOK;
    position.board[1] = BLACK | KNIGHT;
    position.board[2] = BLACK | BISHOP;
    position.board[3] = BLACK | QUEEN;
    position.board[4] = BLACK | KING;
    position.board[5] = BLACK | BISHOP;
    position.board[6] = BLACK | KNIGHT;
    position.board[7] = BLACK | ROOK;

    for i in 16..24 {
        position.board[i] = BLACK | PAWN;
    }

    for i in 96..104 {
        position.board[i] = WHITE | PAWN;
    }

    position.board[112] = WHITE | ROOK;
    position.board[113] = WHITE | KNIGHT;
    position.board[114] = WHITE | BISHOP;
    position.board[115] = WHITE | QUEEN;
    position.board[116] = WHITE | KING;
    position.board[117] = WHITE | BISHOP;
    position.board[118] = WHITE | KNIGHT;
    position.board[119] = WHITE | ROOK;
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


fn main() {
    let mut pos = Position { board: [EMPTY; 128] };
    initialize_position(&mut pos);
    // println!("{}", pos.board[0]);
    // assert!(get_piece_color(pos.board[0]) == BLACK);
    // assert!(get_piece_type(pos.board[0]) == ROOK);
    // let piece = pos.board[112];
    // println!("{}", get_piece_char(piece));
    print_position(pos);
}
