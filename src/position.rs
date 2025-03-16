use crate::movegen::{Move, is_off_board, print_move};
use crate::piece::*;

#[derive(Clone, Copy)]
pub struct Position {
    pub board: [u8; 128],
    pub is_white_turn: bool,
    pub enpassant_square: Option<usize>,
    // white kingside, white queenside, black kingside, black queenside
    pub castling_rights: [bool; 4],
}

impl Position {
    pub fn print(&self) {
        let side_to_move = match self.is_white_turn {
            true => "White",
            false => "Black",
        };
        print!("{} to move ", side_to_move);
        print!("castling {:?}", self.castling_rights);

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
            enpassant_square: None,
            castling_rights: [false, false, false, false],
        };
        let fen_parts = fen_string.split(" ").collect::<Vec<&str>>();
        // currently using only the piece placement, later use side, castling, ep, etc.
        let piece_placement = fen_parts[0];
        let side_to_move = fen_parts[1];
        let castling_rights = fen_parts[2];

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

        for c in castling_rights.chars() {
            match c {
                '-' => break,
                'K' => pos.castling_rights[0] = true,
                'Q' => pos.castling_rights[1] = true,
                'k' => pos.castling_rights[2] = true,
                'q' => pos.castling_rights[3] = true,
                _ => panic!("Unexpected castling rights char: {}", c),
            }
        }
        pos
    }

    pub fn generate_moves(&self) -> Vec<Move> {
        crate::movegen::generate_moves(self)
    }

    fn side_has_castling_rights(&self) -> bool {
        if self.is_white_turn {
            self.castling_rights[0] || self.castling_rights[1]
        } else {
            self.castling_rights[2] || self.castling_rights[3]
        }
    }

    fn handle_castling_move(&mut self, move_: &Move) {
        match move_.to {
            118 => {
                self.board[119] = EMPTY;
                self.board[117] = WHITE | ROOK;
                self.castling_rights[0] = false;
                self.castling_rights[1] = false;
            }
            114 => {
                self.board[112] = EMPTY;
                self.board[115] = WHITE | ROOK;
                self.castling_rights[0] = false;
                self.castling_rights[1] = false;
            }
            6 => {
                self.board[7] = EMPTY;
                self.board[5] = BLACK | ROOK;
                self.castling_rights[2] = false;
                self.castling_rights[3] = false;
            }
            2 => {
                self.board[0] = EMPTY;
                self.board[3] = BLACK | ROOK;
                self.castling_rights[2] = false;
                self.castling_rights[3] = false;
            }
            _ => panic!("invalid square to move to"),
        }
    }

    pub fn make_move(&mut self, move_: &Move) {
        let piece = self.board[move_.from];

        if self.side_has_castling_rights() {
            // lose castling rights when king moves
            if get_piece_type(piece) == KING {
                match self.is_white_turn {
                    true => {
                        self.castling_rights[0] = false;
                        self.castling_rights[1] = false;
                    }
                    false => {
                        self.castling_rights[2] = false;
                        self.castling_rights[3] = false;
                    }
                }
            }
            // lose castling rights when rook moves or gets captured
            if move_.from == 112 || move_.to == 112 {
                self.castling_rights[0] = false;
            }
            if move_.from == 119 || move_.to == 119 {
                self.castling_rights[1] = false;
            }

            if move_.from == 7 || move_.to == 7 {
                self.castling_rights[2] = false;
            }
            if move_.from == 0 || move_.to == 0 {
                self.castling_rights[3] = false;
            }
        }

        if move_.is_castling {
            println!("{:?}", move_);
            print_move(&move_);
            self.handle_castling_move(&move_);
        }

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
