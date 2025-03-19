use crate::movegen::{Move, get_square_string, is_off_board};
use crate::piece::*;

#[derive(Clone, Copy)]
pub struct Position {
    pub board: [u8; 128],
    pub is_white_turn: bool,
    pub enpassant_square: Option<usize>,
    // white kingside, white queenside, black kingside, black queenside
    pub castling_rights: [bool; 4],
    pub king_squares: [usize; 2], // white, black
}

impl Position {
    pub fn print(&self) {
        let side_to_move = match self.is_white_turn {
            true => "White",
            false => "Black",
        };
        print!("{} to move", side_to_move);
        print!(" castling {:?}", self.castling_rights);
        print!(" king squares {:?}", self.king_squares);
        let ep_square = match self.enpassant_square {
            Some(square) => get_square_string(square),
            None => "_".to_string(),
        };
        print!(" EP square {}", ep_square);

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
            king_squares: [127, 127], // we dont know yet
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
                if c == 'K' {
                    pos.king_squares[0] = i;
                } else if c == 'k' {
                    pos.king_squares[1] = i;
                }
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

    pub fn generate_pseudo_moves(&self) -> Vec<Move> {
        crate::movegen::generate_pseudo_moves(self)
    }

    pub fn generate_legal_moves(&mut self) -> Vec<Move> {
        crate::movegen::generate_legal_moves(self)
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

    fn revert_castling_move(&mut self, move_: &Move) {
        match move_.to {
            118 => {
                self.board[119] = WHITE | ROOK;
                self.board[117] = EMPTY;
            }
            114 => {
                self.board[112] = WHITE | ROOK;
                self.board[115] = EMPTY;
            }
            6 => {
                self.board[7] = BLACK | ROOK;
                self.board[5] = EMPTY;
            }
            2 => {
                self.board[0] = BLACK | ROOK;
                self.board[3] = EMPTY;
            }
            _ => panic!("invalid square to move to"),
        }
    }

    pub fn make_move(&mut self, move_: &Move) {
        let piece = self.board[move_.from];

        self.enpassant_square = None;

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
        }
        // lose castling rights when rook moves or gets captured
        if move_.from == 119 || move_.to == 119 {
            self.castling_rights[0] = false;
        }
        if move_.from == 112 || move_.to == 112 {
            self.castling_rights[1] = false;
        }

        if move_.from == 7 || move_.to == 7 {
            self.castling_rights[2] = false;
        }
        if move_.from == 0 || move_.to == 0 {
            self.castling_rights[3] = false;
        }

        if move_.is_castling {
            self.handle_castling_move(&move_);
        }

        if get_piece_type(piece) == KING {
            match self.is_white_turn {
                true => self.king_squares[0] = move_.to,
                false => self.king_squares[1] = move_.to,
            }
        }

        if move_.is_double_pawn {
            for dir in [-1, 1] {
                let square_to_check = move_.to.wrapping_add_signed(dir);
                if is_off_board(square_to_check) {
                    continue;
                }
                let target_piece = self.board[square_to_check];
                if self.is_white_turn && target_piece == BLACK | PAWN {
                    self.enpassant_square = Some(move_.to.wrapping_add(16));
                }
                if !self.is_white_turn && target_piece == WHITE | PAWN {
                    self.enpassant_square = Some(move_.to.wrapping_sub(16));
                }
            }
        }
        if move_.is_enpassant {
            if self.is_white_turn {
                self.board[move_.to + 16] = EMPTY;
            } else {
                self.board[move_.to - 16] = EMPTY;
            }
        }
        if move_.promoted_piece.is_some() {
            self.board[move_.to] = move_.promoted_piece.unwrap();
        } else {
            self.board[move_.to] = piece;
        }

        self.board[move_.from] = EMPTY;
        self.is_white_turn = !self.is_white_turn;
    }

    pub fn unmake_move(
        &mut self,
        move_: &Move,
        piece_at_target: u8,
        original_castling_rights: [bool; 4],
        original_king_squares: [usize; 2],
        original_ep_square: Option<usize>,
    ) {
        if move_.is_castling {
            self.revert_castling_move(&move_);
        }
        let piece = self.board[move_.to];
        self.board[move_.from] = piece;
        self.board[move_.to] = piece_at_target;
        self.is_white_turn = !self.is_white_turn;
        if move_.is_enpassant {
            if self.is_white_turn {
                self.board[move_.to + 16] = BLACK | PAWN;
            } else {
                self.board[move_.to - 16] = WHITE | PAWN;
            }
        }
        if move_.promoted_piece.is_some() {
            if self.is_white_turn {
                self.board[move_.from] = WHITE | PAWN;
            } else {
                self.board[move_.from] = BLACK | PAWN;
            }
        }
        self.castling_rights = original_castling_rights;
        // TODO: Fix unmakemove - for example king squares, do proper reverse
        self.king_squares = original_king_squares;
        self.enpassant_square = original_ep_square;
    }
}
