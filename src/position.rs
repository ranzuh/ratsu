use crate::movegen::{Move, is_off_board};
use crate::piece::{EMPTY, get_piece_char, piece_from_char};

#[derive(Clone, Copy)]
pub struct Position {
    pub board: [u8; 128],
    pub is_white_turn: bool,
}

impl Position {
    pub fn print(&self) {
        let side_to_move = match self.is_white_turn {
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
            print!("{} ", get_piece_char(self.board[i]));
        }
        println!("\n  a b c d e f g h");
    }

    pub fn from_fen(fen_string: &str) -> Self {
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
            } else if c == '/' {
                i += 8;
            } else {
                let piece = piece_from_char(c);
                pos.board[i] = piece;
                i += 1;
            }
        }
        pos
    }

    pub fn generate_moves(&self) -> Vec<Move> {
        crate::movegen::generate_moves(self)
    }

    pub fn make_move(&mut self, move_: &Move) {
        let piece = self.board[move_.from];
        self.board[move_.to] = piece;
        self.board[move_.from] = EMPTY;
        self.is_white_turn = !self.is_white_turn;
    }

    pub fn unmake_move(&mut self, move_: &Move, piece_at_target: u8) {
        let piece = self.board[move_.to];
        self.board[move_.from] = piece;
        self.board[move_.to] = piece_at_target;
        self.is_white_turn = !self.is_white_turn;
    }
}
