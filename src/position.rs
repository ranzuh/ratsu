use crate::evaluation::{get_piece_material_score, get_piece_table_score};
use crate::hash::ZobristKeys;
use crate::movegen::{BOARD_SQUARES, Move, get_file, get_rank, get_square_string, is_off_board};
use crate::piece::*;

pub fn get_square_in_64(square_in_128: usize) -> usize {
    get_rank(square_in_128) * 8 + get_file(square_in_128)
}

fn sq128_to_sq64(sq88: usize) -> usize {
    (7 - (sq88 >> 4)) * 8 + (sq88 & 7)
}

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
    pub bb_color: [u64; 2],
    pub bb_piece: [u64; 6],
    pub material_score: i32,
    pub pst_score: i32,

    prev_target_piece: [u8; 64],
    prev_castling_rights: [[bool; 4]; 64],
    prev_king_squares: [[usize; 2]; 64],
    prev_ep_square: [Option<usize>; 64],
    prev_hash: [u64; 64],
    prev_fifty: [u8; 64],
    prev_material_score: [i32; 64],
    prev_pst_score: [i32; 64],
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

    pub fn from_fen(fen_string: &str) -> Self {
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
            bb_color: [0u64; 2],
            bb_piece: [0u64; 6],
            material_score: 0,
            pst_score: 0,

            prev_target_piece: [0u8; 64],
            prev_castling_rights: [[false, false, false, false]; 64],
            prev_king_squares: [[127, 127]; 64],
            prev_ep_square: [None; 64],
            prev_hash: [0u64; 64],
            prev_fifty: [0u8; 64],
            prev_material_score: [0i32; 64],
            prev_pst_score: [0i32; 64],
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

                pos.bb_add_piece_to(piece, i);
                pos.material_score += get_piece_material_score(piece);
                pos.pst_score += get_piece_table_score(i, piece, get_piece_type(piece));

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
        // castling rights removal is handled when king moves
        match move_.to {
            118 => {
                self.board[119] = EMPTY;
                self.board[117] = WHITE | ROOK;
                self.bb_remove_piece_from(WHITE | ROOK, 119);
                self.bb_add_piece_to(WHITE | ROOK, 117);
                self.pst_score -= get_piece_table_score(119, WHITE | ROOK, ROOK);
                self.pst_score += get_piece_table_score(117, WHITE | ROOK, ROOK);
                self.hash ^= self.piece_hash(119, WHITE | ROOK);
                self.hash ^= self.piece_hash(117, WHITE | ROOK);
            }
            114 => {
                self.board[112] = EMPTY;
                self.board[115] = WHITE | ROOK;
                self.bb_remove_piece_from(WHITE | ROOK, 112);
                self.bb_add_piece_to(WHITE | ROOK, 115);
                self.pst_score -= get_piece_table_score(112, WHITE | ROOK, ROOK);
                self.pst_score += get_piece_table_score(115, WHITE | ROOK, ROOK);
                self.hash ^= self.piece_hash(112, WHITE | ROOK);
                self.hash ^= self.piece_hash(115, WHITE | ROOK);
            }
            6 => {
                self.board[7] = EMPTY;
                self.board[5] = BLACK | ROOK;
                self.bb_remove_piece_from(BLACK | ROOK, 7);
                self.bb_add_piece_to(BLACK | ROOK, 5);
                self.pst_score -= get_piece_table_score(7, BLACK | ROOK, ROOK);
                self.pst_score += get_piece_table_score(5, BLACK | ROOK, ROOK);
                self.hash ^= self.piece_hash(7, BLACK | ROOK);
                self.hash ^= self.piece_hash(5, BLACK | ROOK);
            }
            2 => {
                self.board[0] = EMPTY;
                self.board[3] = BLACK | ROOK;
                self.bb_remove_piece_from(BLACK | ROOK, 0);
                self.bb_add_piece_to(BLACK | ROOK, 3);
                self.pst_score -= get_piece_table_score(0, BLACK | ROOK, ROOK);
                self.pst_score += get_piece_table_score(3, BLACK | ROOK, ROOK);
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
                self.bb_remove_piece_from(WHITE | ROOK, 117);
                self.bb_add_piece_to(WHITE | ROOK, 119);
            }
            114 => {
                self.board[112] = WHITE | ROOK;
                self.board[115] = EMPTY;
                self.bb_remove_piece_from(WHITE | ROOK, 115);
                self.bb_add_piece_to(WHITE | ROOK, 112);
            }
            6 => {
                self.board[7] = BLACK | ROOK;
                self.board[5] = EMPTY;
                self.bb_remove_piece_from(BLACK | ROOK, 5);
                self.bb_add_piece_to(BLACK | ROOK, 7);
            }
            2 => {
                self.board[0] = BLACK | ROOK;
                self.board[3] = EMPTY;
                self.bb_remove_piece_from(BLACK | ROOK, 3);
                self.bb_add_piece_to(BLACK | ROOK, 0);
            }
            _ => panic!("invalid square to move to"),
        }
    }

    fn bb_remove_piece_from(&mut self, piece: u8, square: usize) {
        assert!(piece != EMPTY);
        let bb_bit = 1u64 << sq128_to_sq64(square);
        let bb_side = ((get_piece_color(piece) / 8) - 1) as usize;
        let bb_piece_type = (get_piece_type(piece) - 1) as usize;
        self.bb_color[bb_side] &= !bb_bit;
        self.bb_piece[bb_piece_type] &= !bb_bit;
    }

    fn bb_add_piece_to(&mut self, piece: u8, square: usize) {
        assert!(piece != EMPTY);
        let bb_bit = 1u64 << sq128_to_sq64(square);
        let bb_side = ((get_piece_color(piece) / 8) - 1) as usize;
        let bb_piece_type = (get_piece_type(piece) - 1) as usize;
        self.bb_color[bb_side] |= bb_bit;
        self.bb_piece[bb_piece_type] |= bb_bit;
    }

    pub fn make_move(&mut self, move_: &Move, ply: u32) {
        self.prev_target_piece[ply as usize] = self.board[move_.to];
        self.prev_castling_rights[ply as usize] = self.castling_rights;
        self.prev_king_squares[ply as usize] = self.king_squares;
        self.prev_ep_square[ply as usize] = self.enpassant_square;
        self.prev_hash[ply as usize] = self.hash;
        self.prev_fifty[ply as usize] = self.fifty;
        self.prev_material_score[ply as usize] = self.material_score;
        self.prev_pst_score[ply as usize] = self.pst_score;

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
            self.handle_castling_move(move_);
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
                self.bb_remove_piece_from(BLACK | PAWN, move_.to + 16);
                self.pst_score -= get_piece_table_score(move_.to + 16, BLACK | PAWN, PAWN);
                self.material_score -= get_piece_material_score(BLACK | PAWN);
                self.hash ^= self.piece_hash(move_.to + 16, BLACK | PAWN);
            } else {
                self.board[move_.to - 16] = EMPTY;
                self.bb_remove_piece_from(WHITE | PAWN, move_.to - 16);
                self.pst_score -= get_piece_table_score(move_.to - 16, WHITE | PAWN, PAWN);
                self.material_score -= get_piece_material_score(WHITE | PAWN);
                self.hash ^= self.piece_hash(move_.to - 16, WHITE | PAWN);
            }
        }

        if move_.is_capture {
            let target_piece = self.board[move_.to];
            if target_piece != EMPTY {
                self.bb_remove_piece_from(target_piece, move_.to);
                self.pst_score -=
                    get_piece_table_score(move_.to, target_piece, get_piece_type(target_piece));
                self.material_score -= get_piece_material_score(target_piece);
            }
            self.hash ^= self.piece_hash(move_.to, target_piece);
            self.fifty = 0;
        }

        if let Some(prom_piece) = move_.promoted_piece {
            self.board[move_.to] = prom_piece;
            self.bb_add_piece_to(prom_piece, move_.to);
            self.pst_score +=
                get_piece_table_score(move_.to, prom_piece, get_piece_type(prom_piece));
            self.material_score += get_piece_material_score(prom_piece);
            self.material_score -= get_piece_material_score(piece);
            self.hash ^= self.piece_hash(move_.to, prom_piece);
        } else {
            self.board[move_.to] = piece;
            self.bb_add_piece_to(piece, move_.to);
            self.pst_score += get_piece_table_score(move_.to, piece, piece_type);
            self.hash ^= self.piece_hash(move_.to, piece);
        }

        self.board[move_.from] = EMPTY;
        self.bb_remove_piece_from(piece, move_.from);
        self.pst_score -= get_piece_table_score(move_.from, piece, piece_type);
        self.hash ^= self.piece_hash(move_.from, piece);
        self.is_white_turn = !self.is_white_turn;
        self.hash ^= self.keys.black_to_move_key;
    }

    pub fn unmake_move(&mut self, move_: &Move, ply: u32) {
        if move_.is_castling {
            self.revert_castling_move(move_);
        }
        let mut piece = self.board[move_.to];
        self.bb_remove_piece_from(piece, move_.to);

        self.board[move_.to] = self.prev_target_piece[ply as usize];

        if self.prev_target_piece[ply as usize] != EMPTY {
            self.bb_add_piece_to(self.prev_target_piece[ply as usize], move_.to);
        }

        self.is_white_turn = !self.is_white_turn;
        if move_.is_enpassant {
            if self.is_white_turn {
                self.board[move_.to + 16] = BLACK | PAWN;
                self.bb_add_piece_to(BLACK | PAWN, move_.to + 16);
            } else {
                self.board[move_.to - 16] = WHITE | PAWN;
                self.bb_add_piece_to(WHITE | PAWN, move_.to - 16);
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
        self.bb_add_piece_to(piece, move_.from);

        self.castling_rights = self.prev_castling_rights[ply as usize];
        self.king_squares = self.prev_king_squares[ply as usize];
        self.enpassant_square = self.prev_ep_square[ply as usize];
        self.hash = self.prev_hash[ply as usize];
        self.fifty = self.prev_fifty[ply as usize];
        self.material_score = self.prev_material_score[ply as usize];
        self.pst_score = self.prev_pst_score[ply as usize];
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
