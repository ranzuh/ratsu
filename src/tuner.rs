use crate::{
    bitboard::{NOT_A, NOT_H, RANK_2, RANK_7, adjacent_files, file_fill, north_fill, south_fill},
    evaluation::flip_board,
    piece::{
        BISHOP, BLACK, EMPTY, KING, KNIGHT, PAWN, QUEEN, ROOK, WHITE, get_piece_color,
        get_piece_type, piece_from_char,
    },
    position::{get_phase_value, get_square_in_64, sq88_to_bb},
};

use pyo3::prelude::*;
use rayon::prelude::*;

struct LightPosition {
    board: [u8; 128],
    is_white_turn: bool,
    bb_color: [u64; 2],
    bb_piece: [u64; 6],
    pieces: Vec<(usize, u8, u8)>, // (square, piece, piece_type)
    phase_value: i32,
}

impl LightPosition {
    pub fn from_fen(fen_string: &str) -> Self {
        let mut pos = LightPosition {
            board: [EMPTY; 128],
            is_white_turn: false,
            bb_color: [0u64; 2],
            bb_piece: [0u64; 6],
            pieces: Vec::with_capacity(32),
            phase_value: 0,
        };
        let fen_parts = fen_string.split(" ").collect::<Vec<&str>>();
        pos.is_white_turn = fen_parts[1] == "w";

        let mut i: usize = 0;
        for c in fen_parts[0].chars() {
            if c.is_numeric() {
                i += c.to_digit(10).unwrap() as usize;
            } else if c == '/' {
                i += 8;
            } else {
                let piece = piece_from_char(c);
                let piece_type = get_piece_type(piece);
                pos.board[i] = piece;
                pos.bb_add_piece_to(piece, i);
                pos.pieces.push((i, piece, piece_type));
                pos.phase_value += get_phase_value(piece);
                i += 1;
            }
        }

        pos
    }

    fn bb_add_piece_to(&mut self, piece: u8, square: usize) {
        assert!(piece != EMPTY);
        let bb_bit = 1u64 << sq88_to_bb(square);
        let bb_side = ((get_piece_color(piece) / 8) - 1) as usize;
        let bb_piece_type = (get_piece_type(piece) - 1) as usize;
        self.bb_color[bb_side] |= bb_bit;
        self.bb_piece[bb_piece_type] |= bb_bit;
    }
}

#[pyclass]
struct EvaluationDataset {
    positions: Vec<LightPosition>,
    results: Vec<f32>,
}

#[pymethods]
impl EvaluationDataset {
    #[new]
    fn new(fens_and_results: Vec<(String, f32)>) -> Self {
        let mut positions = Vec::with_capacity(fens_and_results.len());
        let mut results = Vec::with_capacity(fens_and_results.len());

        for (fen, res) in fens_and_results {
            positions.push(LightPosition::from_fen(&fen));
            results.push(res);
        }
        EvaluationDataset { positions, results }
    }
}

#[pyfunction]
fn eval_fen(fen: &str, weights: Vec<i32>) -> PyResult<i32> {
    let w = Weights::from_vec(&weights);
    let mut pos = LightPosition::from_fen(fen);
    Ok(evaluate_with_weights(&mut pos, &w))
}

#[pyfunction]
fn compute_mse(dataset: &mut EvaluationDataset, weights: Vec<i32>, k: f32) -> PyResult<f32> {
    let w = Weights::from_vec(&weights);

    const SCALE: f64 = 1e9;

    // NOTE: Addition of floats is non-associative.
    // If using parallel iterator like Rayon, special care must be taken
    // to ensure compute_mse is deterministic.
    let total_error_int: i128 = dataset
        .positions
        .par_iter()
        .zip(&dataset.results)
        .map(|(pos, result)| {
            let score_side_to_move = evaluate_with_weights(pos, &w) as f32;

            let score = if pos.is_white_turn {
                score_side_to_move
            } else {
                -score_side_to_move
            };
            let sigmoid = 1.0 / (1.0 + 10f32.powf(-k * score / 400.0));

            let error = (result - sigmoid).powi(2);

            (error as f64 * SCALE) as i128
        })
        .sum();

    let final_mse = (total_error_int as f64 / SCALE) / dataset.positions.len() as f64;

    Ok(final_mse as f32)
}

#[pymodule]
fn ratsu(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<EvaluationDataset>()?;
    m.add_function(wrap_pyfunction!(eval_fen, m)?)?;
    m.add_function(wrap_pyfunction!(compute_mse, m)?)?;
    Ok(())
}

#[derive(Debug, PartialEq)]
struct Weights {
    // Material
    material_pawn: i32,
    material_knight: i32,
    material_bishop: i32,
    material_rook: i32,
    material_queen: i32,
    material_king: i32,
    // Piece-square tables
    pawn_mg_pst: [i32; 64],
    knight_mg_pst: [i32; 64],
    bishop_mg_pst: [i32; 64],
    rook_mg_pst: [i32; 64],
    queen_mg_pst: [i32; 64],
    king_mg_pst: [i32; 64],
    pawn_eg_pst: [i32; 64],
    knight_eg_pst: [i32; 64],
    bishop_eg_pst: [i32; 64],
    rook_eg_pst: [i32; 64],
    queen_eg_pst: [i32; 64],
    king_eg_pst: [i32; 64],
    // Precomputed flipped PSTs for black
    pawn_mg_flip: [i32; 64],
    knight_mg_flip: [i32; 64],
    bishop_mg_flip: [i32; 64],
    rook_mg_flip: [i32; 64],
    queen_mg_flip: [i32; 64],
    king_mg_flip: [i32; 64],
    pawn_eg_flip: [i32; 64],
    knight_eg_flip: [i32; 64],
    bishop_eg_flip: [i32; 64],
    rook_eg_flip: [i32; 64],
    queen_eg_flip: [i32; 64],
    king_eg_flip: [i32; 64],
    // Positional bonuses/penalties
    doubled_pawn_penalty: i32,
    isolated_pawn_penalty: i32,
    backwards_pawn_penalty: i32,
    passed_pawn_bonus: i32,
    rook_semi_open_file_bonus: i32,
    rook_open_file_bonus: i32,
    rook_on_seventh_bonus: i32,
    bishop_pair_bonus: i32,
}

impl Weights {
    #[cfg(test)]
    fn to_vec(&self) -> Vec<i32> {
        let mut v = vec![
            self.material_pawn,
            self.material_knight,
            self.material_bishop,
            self.material_rook,
            self.material_queen,
            self.material_king,
            self.doubled_pawn_penalty,
            self.isolated_pawn_penalty,
            self.backwards_pawn_penalty,
            self.passed_pawn_bonus,
            self.rook_semi_open_file_bonus,
            self.rook_open_file_bonus,
            self.rook_on_seventh_bonus,
            self.bishop_pair_bonus,
        ];
        v.extend_from_slice(&self.pawn_mg_pst);
        v.extend_from_slice(&self.knight_mg_pst);
        v.extend_from_slice(&self.bishop_mg_pst);
        v.extend_from_slice(&self.rook_mg_pst);
        v.extend_from_slice(&self.queen_mg_pst);
        v.extend_from_slice(&self.king_mg_pst);
        v.extend_from_slice(&self.pawn_eg_pst);
        v.extend_from_slice(&self.knight_eg_pst);
        v.extend_from_slice(&self.bishop_eg_pst);
        v.extend_from_slice(&self.rook_eg_pst);
        v.extend_from_slice(&self.queen_eg_pst);
        v.extend_from_slice(&self.king_eg_pst);
        v
    }

    fn from_vec(v: &[i32]) -> Self {
        let pawn_mg_pst: [i32; 64] = v[14..78].try_into().unwrap();
        let knight_mg_pst: [i32; 64] = v[78..142].try_into().unwrap();
        let bishop_mg_pst: [i32; 64] = v[142..206].try_into().unwrap();
        let rook_mg_pst: [i32; 64] = v[206..270].try_into().unwrap();
        let queen_mg_pst: [i32; 64] = v[270..334].try_into().unwrap();
        let king_mg_pst: [i32; 64] = v[334..398].try_into().unwrap();
        let pawn_eg_pst: [i32; 64] = v[398..462].try_into().unwrap();
        let knight_eg_pst: [i32; 64] = v[462..526].try_into().unwrap();
        let bishop_eg_pst: [i32; 64] = v[526..590].try_into().unwrap();
        let rook_eg_pst: [i32; 64] = v[590..654].try_into().unwrap();
        let queen_eg_pst: [i32; 64] = v[654..718].try_into().unwrap();
        let king_eg_pst: [i32; 64] = v[718..782].try_into().unwrap();
        Self {
            material_pawn: v[0],
            material_knight: v[1],
            material_bishop: v[2],
            material_rook: v[3],
            material_queen: v[4],
            material_king: v[5],
            doubled_pawn_penalty: v[6],
            isolated_pawn_penalty: v[7],
            backwards_pawn_penalty: v[8],
            passed_pawn_bonus: v[9],
            rook_semi_open_file_bonus: v[10],
            rook_open_file_bonus: v[11],
            rook_on_seventh_bonus: v[12],
            bishop_pair_bonus: v[13],
            pawn_mg_flip: flip_board(&pawn_mg_pst),
            knight_mg_flip: flip_board(&knight_mg_pst),
            bishop_mg_flip: flip_board(&bishop_mg_pst),
            rook_mg_flip: flip_board(&rook_mg_pst),
            queen_mg_flip: flip_board(&queen_mg_pst),
            king_mg_flip: flip_board(&king_mg_pst),
            pawn_eg_flip: flip_board(&pawn_eg_pst),
            knight_eg_flip: flip_board(&knight_eg_pst),
            bishop_eg_flip: flip_board(&bishop_eg_pst),
            rook_eg_flip: flip_board(&rook_eg_pst),
            queen_eg_flip: flip_board(&queen_eg_pst),
            king_eg_flip: flip_board(&king_eg_pst),
            pawn_mg_pst,
            knight_mg_pst,
            bishop_mg_pst,
            rook_mg_pst,
            queen_mg_pst,
            king_mg_pst,
            pawn_eg_pst,
            knight_eg_pst,
            bishop_eg_pst,
            rook_eg_pst,
            queen_eg_pst,
            king_eg_pst,
        }
    }
}

fn evaluate_with_weights(position: &LightPosition, weights: &Weights) -> i32 {
    let mut score = 0;
    let side = if position.is_white_turn { 1 } else { -1 };
    let mut mg_pst = 0;
    let mut eg_pst = 0;

    for &(square, piece, piece_type) in &position.pieces {
        mg_pst += get_mg_piece_table_score_with_weights(square, piece, piece_type, weights);
        eg_pst += get_eg_piece_table_score_with_weights(square, piece, piece_type, weights);
        score += get_piece_material_score_with_weights(piece, weights);
    }

    let mg_phase = position.phase_value;
    let eg_phase = 6400 - mg_phase;
    score += (mg_pst * mg_phase + eg_pst * eg_phase) / 6400;
    score += bb_pawn_structure_with_weights(&position.bb_color, &position.bb_piece, weights);
    score += bb_rook_score_with_weights(&position.bb_color, &position.bb_piece, weights);

    let white_bishops = (position.bb_color[0] & position.bb_piece[2]).count_ones();
    let black_bishops = (position.bb_color[1] & position.bb_piece[2]).count_ones();
    if white_bishops >= 2 {
        score += weights.bishop_pair_bonus;
    }
    if black_bishops >= 2 {
        score -= weights.bishop_pair_bonus;
    }

    score * side
}

fn get_mg_piece_table_score_with_weights(
    square: usize,
    piece: u8,
    piece_type: u8,
    weights: &Weights,
) -> i32 {
    let sq = get_square_in_64(square);
    if get_piece_color(piece) == WHITE {
        match piece_type {
            PAWN => weights.pawn_mg_pst[sq],
            KNIGHT => weights.knight_mg_pst[sq],
            BISHOP => weights.bishop_mg_pst[sq],
            ROOK => weights.rook_mg_pst[sq],
            QUEEN => weights.queen_mg_pst[sq],
            KING => weights.king_mg_pst[sq],
            _ => panic!("Unexpected piece {}", piece),
        }
    } else {
        -match piece_type {
            PAWN => weights.pawn_mg_flip[sq],
            KNIGHT => weights.knight_mg_flip[sq],
            BISHOP => weights.bishop_mg_flip[sq],
            ROOK => weights.rook_mg_flip[sq],
            QUEEN => weights.queen_mg_flip[sq],
            KING => weights.king_mg_flip[sq],
            _ => panic!("Unexpected piece {}", piece),
        }
    }
}

fn get_eg_piece_table_score_with_weights(
    square: usize,
    piece: u8,
    piece_type: u8,
    weights: &Weights,
) -> i32 {
    let sq = get_square_in_64(square);
    if get_piece_color(piece) == WHITE {
        match piece_type {
            PAWN => weights.pawn_eg_pst[sq],
            KNIGHT => weights.knight_eg_pst[sq],
            BISHOP => weights.bishop_eg_pst[sq],
            ROOK => weights.rook_eg_pst[sq],
            QUEEN => weights.queen_eg_pst[sq],
            KING => weights.king_eg_pst[sq],
            _ => panic!("Unexpected piece {}", piece),
        }
    } else {
        -match piece_type {
            PAWN => weights.pawn_eg_flip[sq],
            KNIGHT => weights.knight_eg_flip[sq],
            BISHOP => weights.bishop_eg_flip[sq],
            ROOK => weights.rook_eg_flip[sq],
            QUEEN => weights.queen_eg_flip[sq],
            KING => weights.king_eg_flip[sq],
            _ => panic!("Unexpected piece {}", piece),
        }
    }
}

fn get_material_score_with_weights(piece: u8, weights: &Weights) -> i32 {
    match get_piece_type(piece) {
        PAWN => weights.material_pawn,
        KNIGHT => weights.material_knight,
        BISHOP => weights.material_bishop,
        ROOK => weights.material_rook,
        QUEEN => weights.material_queen,
        KING => weights.material_king,
        EMPTY => 0,
        _ => panic!("{}", get_piece_type(piece)),
    }
}

fn get_piece_material_score_with_weights(piece: u8, weights: &Weights) -> i32 {
    let side = match get_piece_color(piece) {
        WHITE => 1,
        BLACK => -1,
        EMPTY => 0,
        _ => panic!("{}", get_piece_color(piece)),
    };
    let material_score = get_material_score_with_weights(piece, weights);
    side * material_score
}

fn bb_pawn_structure_with_weights(
    bb_color: &[u64; 2],
    bb_piece: &[u64; 6],
    weights: &Weights,
) -> i32 {
    let white_pawns = bb_color[0] & bb_piece[0];
    let black_pawns = bb_color[1] & bb_piece[0];

    let mut score = 0;

    // double pawns
    let white_doubled = white_pawns & north_fill(white_pawns << 8);
    let black_doubled = black_pawns & south_fill(black_pawns >> 8);

    score -= weights.doubled_pawn_penalty * white_doubled.count_ones() as i32;
    score += weights.doubled_pawn_penalty * black_doubled.count_ones() as i32;

    // isolated pawns
    let white_isolated = white_pawns & !adjacent_files(file_fill(white_pawns));
    let black_isolated = black_pawns & !adjacent_files(file_fill(black_pawns));

    score -= weights.isolated_pawn_penalty * white_isolated.count_ones() as i32;
    score += weights.isolated_pawn_penalty * black_isolated.count_ones() as i32;

    // passed pawns
    let b_south = south_fill(black_pawns);
    let b_sentinel = b_south | adjacent_files(south_fill(black_pawns >> 8));
    let mut white_passed = white_pawns & !b_sentinel;
    while white_passed != 0 {
        let sq = white_passed.trailing_zeros() as i32;
        let rank = sq / 8; // 0=rank1, 7=rank8
        score += rank * weights.passed_pawn_bonus;
        white_passed &= white_passed - 1;
    }

    let w_north = north_fill(white_pawns);
    let w_sentinel = w_north | adjacent_files(north_fill(white_pawns << 8));
    let mut black_passed = black_pawns & !w_sentinel;
    while black_passed != 0 {
        let sq = black_passed.trailing_zeros() as i32;
        let rank = sq / 8;
        score -= (7 - rank) * weights.passed_pawn_bonus;
        black_passed &= black_passed - 1;
    }

    // backwards pawns
    let white_stop = white_pawns << 8; // square in front of each pawn
    let black_attacks = ((black_pawns & NOT_A) >> 9) | ((black_pawns & NOT_H) >> 7);
    let white_behind = white_pawns & !south_fill(adjacent_files(white_pawns));
    let white_backward = white_behind & !white_isolated & (white_stop & black_attacks) >> 8;

    let black_stop = black_pawns >> 8; // square in front of each pawn
    let white_attacks = ((white_pawns & NOT_A) << 7) | ((white_pawns & NOT_H) << 9);
    let black_behind = black_pawns & !north_fill(adjacent_files(black_pawns));
    let black_backward = black_behind & !black_isolated & (black_stop & white_attacks) << 8;

    score -= weights.backwards_pawn_penalty * white_backward.count_ones() as i32;
    score += weights.backwards_pawn_penalty * black_backward.count_ones() as i32;

    score
}

fn bb_rook_score_with_weights(bb_color: &[u64; 2], bb_piece: &[u64; 6], weights: &Weights) -> i32 {
    let white_pawns = bb_color[0] & bb_piece[0];
    let black_pawns = bb_color[1] & bb_piece[0];
    let white_rooks = bb_color[0] & bb_piece[3];
    let black_rooks = bb_color[1] & bb_piece[3];

    let w_pawn_files = file_fill(white_pawns);
    let b_pawn_files = file_fill(black_pawns);

    let open_files = !w_pawn_files & !b_pawn_files;
    let w_semi_open = !w_pawn_files & b_pawn_files;
    let b_semi_open = !b_pawn_files & w_pawn_files;

    let mut score = 0;

    // White rooks
    score += weights.rook_open_file_bonus * (white_rooks & open_files).count_ones() as i32;
    score += weights.rook_semi_open_file_bonus * (white_rooks & w_semi_open).count_ones() as i32;
    score += weights.rook_on_seventh_bonus * (white_rooks & RANK_7).count_ones() as i32;

    // Black rooks
    score -= weights.rook_open_file_bonus * (black_rooks & open_files).count_ones() as i32;
    score -= weights.rook_semi_open_file_bonus * (black_rooks & b_semi_open).count_ones() as i32;
    score -= weights.rook_on_seventh_bonus * (black_rooks & RANK_2).count_ones() as i32;

    score
}

#[cfg(test)]
mod tests {
    use crate::START_POSITION_FEN;
    use crate::evaluation::{
        BACKWARDS_PAWN_PENALTY, BISHOP_EG_PST, BISHOP_MG_PST, BISHOP_PAIR_BONUS,
        DOUBLED_PAWN_PENALTY, ISOLATED_PAWN_PENALTY, KING_EG_PST, KING_MG_PST, KNIGHT_EG_PST,
        KNIGHT_MG_PST, MATERIAL_BISHOP, MATERIAL_KING, MATERIAL_KNIGHT, MATERIAL_PAWN,
        MATERIAL_QUEEN, MATERIAL_ROOK, PASSED_PAWN_BONUS, PAWN_EG_PST, PAWN_MG_PST, QUEEN_MG_PST,
        ROOK_EG_PST, ROOK_MG_PST, ROOK_ON_SEVENTH_BONUS, ROOK_OPEN_FILE_BONUS,
        ROOK_SEMI_OPEN_FILE_BONUS, evaluate,
    };
    use crate::position::Position;

    use super::*;

    const DEFAULT_WEIGHTS: Weights = Weights {
        material_pawn: MATERIAL_PAWN,
        material_knight: MATERIAL_KNIGHT,
        material_bishop: MATERIAL_BISHOP,
        material_rook: MATERIAL_ROOK,
        material_queen: MATERIAL_QUEEN,
        material_king: MATERIAL_KING,
        pawn_mg_pst: PAWN_MG_PST,
        knight_mg_pst: KNIGHT_MG_PST,
        bishop_mg_pst: BISHOP_MG_PST,
        rook_mg_pst: ROOK_MG_PST,
        queen_mg_pst: PAWN_MG_PST,
        king_mg_pst: KING_MG_PST,
        pawn_eg_pst: PAWN_EG_PST,
        knight_eg_pst: KNIGHT_EG_PST,
        bishop_eg_pst: BISHOP_EG_PST,
        rook_eg_pst: ROOK_EG_PST,
        queen_eg_pst: QUEEN_MG_PST,
        king_eg_pst: KING_EG_PST,
        doubled_pawn_penalty: DOUBLED_PAWN_PENALTY,
        isolated_pawn_penalty: ISOLATED_PAWN_PENALTY,
        backwards_pawn_penalty: BACKWARDS_PAWN_PENALTY,
        passed_pawn_bonus: PASSED_PAWN_BONUS,
        rook_semi_open_file_bonus: ROOK_SEMI_OPEN_FILE_BONUS,
        rook_open_file_bonus: ROOK_OPEN_FILE_BONUS,
        rook_on_seventh_bonus: ROOK_ON_SEVENTH_BONUS,
        bishop_pair_bonus: BISHOP_PAIR_BONUS,
    };

    #[test]
    fn test_evaluate_with_weights() {
        let pos = Position::from_fen(START_POSITION_FEN);
        let eval = evaluate(&pos);
        let mut light_pos = LightPosition::from_fen(START_POSITION_FEN);
        let eval_with_default_weights = evaluate_with_weights(&mut light_pos, &DEFAULT_WEIGHTS);
        assert_eq!(eval, eval_with_default_weights);

        let pawn_pos = Position::from_fen("4k3/1p2p3/4p1P1/3p4/3P4/4P2p/1P2P3/4K3 w - - 0 1");
        let pawn_eval = evaluate(&pawn_pos);
        let mut light_pawn_pos =
            LightPosition::from_fen("4k3/1p2p3/4p1P1/3p4/3P4/4P2p/1P2P3/4K3 w - - 0 1");
        let pawn_eval_with_default_weights =
            evaluate_with_weights(&mut light_pawn_pos, &DEFAULT_WEIGHTS);
        assert_eq!(pawn_eval, pawn_eval_with_default_weights);

        let rook_pos = Position::from_fen("2r1kr2/Rp1p1p2/8/8/8/8/rP1P2P1/2R1K1R1 w - - 0 1");
        let rook_eval = evaluate(&rook_pos);
        let mut light_rook_pos =
            LightPosition::from_fen("2r1kr2/Rp1p1p2/8/8/8/8/rP1P2P1/2R1K1R1 w - - 0 1");
        let rook_eval_with_default_weights =
            evaluate_with_weights(&mut light_rook_pos, &DEFAULT_WEIGHTS);
        assert_eq!(rook_eval, rook_eval_with_default_weights);
    }

    #[test]
    fn test_to_vec_length() {
        let w = DEFAULT_WEIGHTS;
        assert_eq!(w.to_vec().len(), 782); // 14 scalars + 12*64 PST values
    }

    #[test]
    fn test_roundtrip() {
        // Converting to vec and back should give identical weights
        let original = DEFAULT_WEIGHTS;
        let roundtripped = Weights::from_vec(&original.to_vec());
        assert_eq!(original, roundtripped);
    }

    #[test]
    fn test_scalars_are_at_correct_positions() {
        let w = DEFAULT_WEIGHTS;
        let v = w.to_vec();
        assert_eq!(v[0], w.material_pawn);
        assert_eq!(v[1], w.material_knight);
        assert_eq!(v[13], w.bishop_pair_bonus);
    }

    #[test]
    fn test_psts_are_at_correct_positions() {
        let w = DEFAULT_WEIGHTS;
        let v = w.to_vec();
        assert_eq!(v[14..78], w.pawn_mg_pst);
        assert_eq!(v[78..142], w.knight_mg_pst);
        assert_eq!(v[334..398], w.king_mg_pst);
        assert_eq!(v[718..782], w.king_eg_pst);
    }

    #[test]
    fn test_modifying_vec_changes_weights() {
        let mut v = DEFAULT_WEIGHTS.to_vec();
        v[0] = 999; // change material_pawn
        let w = Weights::from_vec(&v);
        assert_eq!(w.material_pawn, 999);
        // everything else unchanged
        assert_eq!(w.material_knight, DEFAULT_WEIGHTS.material_knight);
    }
}
