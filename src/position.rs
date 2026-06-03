use crate::hash::ZobristKeys;
use crate::movegen::{BOARD_SQUARES, Move, get_file, get_rank, get_square_string, is_off_board};
use crate::nnue::{ACC_SIZE, Nnue, get_feature_index};
use crate::piece::*;

pub fn get_square_in_64(square_in_128: usize) -> usize {
    get_rank(square_in_128) * 8 + get_file(square_in_128)
}

pub fn sq88_to_bb(sq88: usize) -> usize {
    (7 - (sq88 >> 4)) * 8 + (sq88 & 7)
}

pub fn get_phase_value(piece: u8) -> i32 {
    match get_piece_type(piece) {
        PAWN => 0,
        KNIGHT => 300,
        BISHOP => 300,
        ROOK => 500,
        QUEEN => 1000,
        KING => 0,
        EMPTY => 0,
        _ => panic!("{}", get_piece_type(piece)),
    }
}

pub const MAX_PLY: usize = 100;

#[derive(Clone)]
pub struct Position {
    pub board: [u8; 128],
    pub is_white_turn: bool,
    pub enpassant_square: Option<usize>,
    // white kingside, white queenside, black kingside, black queenside
    pub castling_rights: [bool; 4],
    pub king_squares: [usize; 2], // white, black
    pub keys: ZobristKeys,
    pub hash: u64,
    pub repetition_stack: [u64; 1024],
    pub repetition_index: usize,
    pub fifty: u8,
    pub acc_white: [i16; ACC_SIZE],
    pub acc_black: [i16; ACC_SIZE],

    prev_target_piece: [u8; MAX_PLY],
    prev_castling_rights: [[bool; 4]; MAX_PLY],
    prev_king_squares: [[usize; 2]; MAX_PLY],
    prev_ep_square: [Option<usize>; MAX_PLY],
    prev_hash: [u64; MAX_PLY],
    prev_fifty: [u8; MAX_PLY],
    prev_acc_white: [[i16; ACC_SIZE]; MAX_PLY],
    prev_acc_black: [[i16; ACC_SIZE]; MAX_PLY],
}

impl Position {
    #[allow(dead_code)]
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
        print!(" hm {}", self.fifty);

        let mut rank = 8;
        for i in BOARD_SQUARES {
            if i % 16 == 0 {
                print!("\n{} ", rank);
                rank -= 1;
            }
            print!("{} ", get_piece_char(self.board[i]));
        }
        println!("\n  a b c d e f g h");
    }

    pub fn is_repetition(&self) -> bool {
        for i in 0..self.repetition_index {
            if self.repetition_stack[i] == self.hash {
                return true;
            }
        }
        false
    }

    fn acc_add_feature(&mut self, piece: u8, square: usize, nnue: &Nnue) {
        let wi = get_feature_index(square, piece, true);
        let bi = get_feature_index(square, piece, false);
        let w_col = &nnue.acc_weights[wi];
        let b_col = &nnue.acc_weights[bi];
        for i in 0..ACC_SIZE {
            self.acc_white[i] += w_col[i];
            self.acc_black[i] += b_col[i];
        }
    }

    fn acc_sub_feature(&mut self, piece: u8, square: usize, nnue: &Nnue) {
        let wi = get_feature_index(square, piece, true);
        let bi = get_feature_index(square, piece, false);
        let w_col = &nnue.acc_weights[wi];
        let b_col = &nnue.acc_weights[bi];
        for i in 0..ACC_SIZE {
            self.acc_white[i] -= w_col[i];
            self.acc_black[i] -= b_col[i];
        }
    }

    fn acc_add_sub_feature(
        &mut self,
        add_piece: u8,
        add_square: usize,
        sub_piece: u8,
        sub_square: usize,
        nnue: &Nnue,
    ) {
        let aw = get_feature_index(add_square, add_piece, true);
        let ab = get_feature_index(add_square, add_piece, false);
        let sw = get_feature_index(sub_square, sub_piece, true);
        let sb = get_feature_index(sub_square, sub_piece, false);
        let aw_col = &nnue.acc_weights[aw];
        let ab_col = &nnue.acc_weights[ab];
        let sw_col = &nnue.acc_weights[sw];
        let sb_col = &nnue.acc_weights[sb];
        for i in 0..ACC_SIZE {
            self.acc_white[i] += aw_col[i] - sw_col[i];
            self.acc_black[i] += ab_col[i] - sb_col[i];
        }
    }

    pub fn from_fen(fen_string: &str, nnue: &Nnue) -> Self {
        let mut pos = Position {
            board: [EMPTY; 128],
            is_white_turn: false,
            enpassant_square: None,
            castling_rights: [false, false, false, false],
            king_squares: [127, 127], // we dont know yet
            keys: ZobristKeys::new(),
            hash: 0u64, // not generated yet
            repetition_stack: [0u64; 1024],
            repetition_index: 0,
            fifty: 0,
            acc_white: [0i16; ACC_SIZE],
            acc_black: [0i16; ACC_SIZE],

            prev_target_piece: [0u8; MAX_PLY],
            prev_castling_rights: [[false, false, false, false]; MAX_PLY],
            prev_king_squares: [[127, 127]; MAX_PLY],
            prev_ep_square: [None; MAX_PLY],
            prev_hash: [0u64; MAX_PLY],
            prev_fifty: [0u8; MAX_PLY],
            prev_acc_white: [[0i16; ACC_SIZE]; MAX_PLY],
            prev_acc_black: [[0i16; ACC_SIZE]; MAX_PLY],
        };

        let fen_parts = fen_string.split(" ").collect::<Vec<&str>>();
        // currently using only the piece placement, later use side, castling, ep, etc.
        let piece_placement = fen_parts[0];
        let side_to_move = fen_parts[1];
        let castling_rights = fen_parts[2];
        let ep_square = fen_parts[3].trim_ascii_end();

        if ep_square != "-" {
            let ep_file = ep_square.chars().next().unwrap() as usize;
            let ep_rank = ep_square.chars().nth(1).unwrap().to_digit(10).unwrap() as usize;
            let ep_square_128 = (ep_file - 97) + ((8 - ep_rank) * 16);
            pos.enpassant_square = Some(ep_square_128);
        }

        pos.is_white_turn = side_to_move == "w";

        pos.acc_white = nnue.acc_bias;
        pos.acc_black = nnue.acc_bias;

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

                pos.acc_add_feature(piece, i, nnue);

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
        pos.generate_hash();
        pos
    }

    fn piece_hash(&self, square: usize, piece: u8) -> u64 {
        // empty squares don't change hash
        if get_piece_type(piece) == EMPTY {
            return 0;
        }
        let square64 = get_square_in_64(square);
        self.keys.piece_keys[square64][piece as usize]
    }

    fn generate_hash(&mut self) {
        // Hash pieces
        for square in BOARD_SQUARES {
            let piece = self.board[square];
            if get_piece_type(piece) != EMPTY {
                self.hash ^= self.piece_hash(square, piece);
            }
        }

        // Hash side to move
        if !self.is_white_turn {
            self.hash ^= self.keys.black_to_move_key;
        }

        // Hash en passant file (if any)
        if let Some(ep_square) = self.enpassant_square {
            let ep_file = get_file(ep_square);
            self.hash ^= self.keys.enpassant_file_keys[ep_file];
        }

        // Hash castling rights
        for (idx, &has_right) in self.castling_rights.iter().enumerate() {
            if has_right {
                self.hash ^= self.keys.castling_rights_keys[idx];
            }
        }
    }

    #[allow(dead_code)]
    pub fn generate_pseudo_moves(&self) -> Vec<Move> {
        crate::movegen::generate_pseudo_moves(self, false)
    }

    pub fn generate_tactical_moves(&self) -> Vec<Move> {
        crate::movegen::generate_pseudo_moves(self, true)
    }

    pub fn generate_legal_moves(&mut self, nnue: &Nnue) -> Vec<Move> {
        crate::movegen::generate_legal_moves(self, nnue)
    }

    fn side_has_castling_rights(&self) -> bool {
        if self.is_white_turn {
            self.castling_rights[0] || self.castling_rights[1]
        } else {
            self.castling_rights[2] || self.castling_rights[3]
        }
    }

    fn handle_castling_move(&mut self, move_: &Move, nnue: &Nnue) {
        // castling rights removal is handled when king moves
        match move_.to {
            118 => {
                self.board[119] = EMPTY;
                self.board[117] = WHITE | ROOK;
                self.acc_add_sub_feature(WHITE | ROOK, 117, WHITE | ROOK, 119, nnue);
                self.hash ^= self.piece_hash(119, WHITE | ROOK);
                self.hash ^= self.piece_hash(117, WHITE | ROOK);
            }
            114 => {
                self.board[112] = EMPTY;
                self.board[115] = WHITE | ROOK;
                self.acc_add_sub_feature(WHITE | ROOK, 115, WHITE | ROOK, 112, nnue);
                self.hash ^= self.piece_hash(112, WHITE | ROOK);
                self.hash ^= self.piece_hash(115, WHITE | ROOK);
            }
            6 => {
                self.board[7] = EMPTY;
                self.board[5] = BLACK | ROOK;
                self.acc_add_sub_feature(BLACK | ROOK, 5, BLACK | ROOK, 7, nnue);
                self.hash ^= self.piece_hash(7, BLACK | ROOK);
                self.hash ^= self.piece_hash(5, BLACK | ROOK);
            }
            2 => {
                self.board[0] = EMPTY;
                self.board[3] = BLACK | ROOK;
                self.acc_add_sub_feature(BLACK | ROOK, 3, BLACK | ROOK, 0, nnue);
                self.hash ^= self.piece_hash(0, BLACK | ROOK);
                self.hash ^= self.piece_hash(3, BLACK | ROOK);
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

    pub fn make_move(&mut self, move_: &Move, ply: u32, nnue: &Nnue) {
        self.prev_target_piece[ply as usize] = self.board[move_.to];
        self.prev_castling_rights[ply as usize] = self.castling_rights;
        self.prev_king_squares[ply as usize] = self.king_squares;
        self.prev_ep_square[ply as usize] = self.enpassant_square;
        self.prev_hash[ply as usize] = self.hash;
        self.prev_fifty[ply as usize] = self.fifty;
        self.prev_acc_white[ply as usize] = self.acc_white;
        self.prev_acc_black[ply as usize] = self.acc_black;

        let piece = self.board[move_.from];
        let piece_type = get_piece_type(piece);
        // remove previous en passant square from hash
        if let Some(ep_square) = self.enpassant_square {
            self.hash ^= self.keys.enpassant_file_keys[get_file(ep_square)];
        }
        self.enpassant_square = None;
        self.fifty += 1;

        if piece_type == PAWN {
            self.fifty = 0;
        }

        if self.side_has_castling_rights() {
            // lose castling rights when king moves
            if piece_type == KING {
                match self.is_white_turn {
                    true => {
                        if self.castling_rights[0] {
                            self.hash ^= self.keys.castling_rights_keys[0];
                            self.castling_rights[0] = false;
                        }
                        if self.castling_rights[1] {
                            self.hash ^= self.keys.castling_rights_keys[1];
                            self.castling_rights[1] = false;
                        }
                    }
                    false => {
                        if self.castling_rights[2] {
                            self.hash ^= self.keys.castling_rights_keys[2];
                            self.castling_rights[2] = false;
                        }
                        if self.castling_rights[3] {
                            self.hash ^= self.keys.castling_rights_keys[3];
                            self.castling_rights[3] = false;
                        }
                    }
                }
            }
        }
        // lose castling rights when rook moves or gets captured
        if self.castling_rights[0] && (move_.from == 119 || move_.to == 119) {
            self.castling_rights[0] = false;
            self.hash ^= self.keys.castling_rights_keys[0];
        }
        if self.castling_rights[1] && (move_.from == 112 || move_.to == 112) {
            self.castling_rights[1] = false;
            self.hash ^= self.keys.castling_rights_keys[1];
        }
        if self.castling_rights[2] && (move_.from == 7 || move_.to == 7) {
            self.castling_rights[2] = false;
            self.hash ^= self.keys.castling_rights_keys[2];
        }
        if self.castling_rights[3] && (move_.from == 0 || move_.to == 0) {
            self.castling_rights[3] = false;
            self.hash ^= self.keys.castling_rights_keys[3];
        }

        if move_.is_castling {
            self.handle_castling_move(move_, nnue);
        }

        if piece_type == KING {
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
                    self.enpassant_square = Some(move_.to + 16);
                    self.hash ^= self.keys.enpassant_file_keys[get_file(move_.to + 16)]
                } else if !self.is_white_turn && target_piece == WHITE | PAWN {
                    self.enpassant_square = Some(move_.to - 16);
                    self.hash ^= self.keys.enpassant_file_keys[get_file(move_.to - 16)]
                }
            }
        }
        if move_.is_enpassant {
            if self.is_white_turn {
                self.board[move_.to + 16] = EMPTY;
                self.acc_sub_feature(BLACK | PAWN, move_.to + 16, nnue);
                self.hash ^= self.piece_hash(move_.to + 16, BLACK | PAWN);
            } else {
                self.board[move_.to - 16] = EMPTY;
                self.acc_sub_feature(WHITE | PAWN, move_.to - 16, nnue);
                self.hash ^= self.piece_hash(move_.to - 16, WHITE | PAWN);
            }
        }

        if move_.is_capture {
            let target_piece = self.board[move_.to];
            if target_piece != EMPTY {
                self.acc_sub_feature(target_piece, move_.to, nnue);
            }
            self.hash ^= self.piece_hash(move_.to, target_piece);
            self.fifty = 0;
        }

        let add_piece;

        if let Some(prom_piece) = move_.promoted_piece {
            self.board[move_.to] = prom_piece;
            add_piece = prom_piece;
            self.hash ^= self.piece_hash(move_.to, prom_piece);
        } else {
            self.board[move_.to] = piece;
            add_piece = piece;
            self.hash ^= self.piece_hash(move_.to, piece);
        }

        self.board[move_.from] = EMPTY;
        self.acc_add_sub_feature(add_piece, move_.to, piece, move_.from, nnue);
        self.hash ^= self.piece_hash(move_.from, piece);
        self.is_white_turn = !self.is_white_turn;
        self.hash ^= self.keys.black_to_move_key;
    }

    pub fn unmake_move(&mut self, move_: &Move, ply: u32) {
        if move_.is_castling {
            self.revert_castling_move(move_);
        }
        let mut piece = self.board[move_.to];

        self.board[move_.to] = self.prev_target_piece[ply as usize];

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
                piece = WHITE | PAWN;
            } else {
                piece = BLACK | PAWN;
            }
        }

        self.board[move_.from] = piece;

        self.castling_rights = self.prev_castling_rights[ply as usize];
        self.king_squares = self.prev_king_squares[ply as usize];
        self.enpassant_square = self.prev_ep_square[ply as usize];
        self.hash = self.prev_hash[ply as usize];
        self.fifty = self.prev_fifty[ply as usize];
        self.acc_white = self.prev_acc_white[ply as usize];
        self.acc_black = self.prev_acc_black[ply as usize];
    }

    pub fn make_null(&mut self) {
        self.is_white_turn = !self.is_white_turn;
        self.hash ^= self.keys.black_to_move_key;

        // hash enpassant if available (remove enpassant square from hash key )
        if let Some(ep_square) = self.enpassant_square {
            self.hash ^= self.keys.enpassant_file_keys[get_file(ep_square)];
        }
        self.enpassant_square = None;
    }

    pub fn unmake_null(&mut self, copy_ep: Option<usize>) {
        self.is_white_turn = !self.is_white_turn;
        self.hash ^= self.keys.black_to_move_key;
        if let Some(ep_square) = copy_ep {
            self.hash ^= self.keys.enpassant_file_keys[get_file(ep_square)];
        }
        self.enpassant_square = copy_ep;
    }
}
