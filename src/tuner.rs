use crate::{
    evaluation::{flip_board, init_pawn_ranks},
    piece::{
        BISHOP, BLACK, EMPTY, KING, KNIGHT, PAWN, QUEEN, ROOK, WHITE, get_piece_color,
        get_piece_type,
    },
    position::{Position, get_square_in_64},
};

use pyo3::prelude::*;
use rayon::prelude::*;

#[pyfunction]
fn eval_fen(fen: &str, weights: Vec<i32>) -> PyResult<i32> {
    let pos = Position::from_fen(fen);
    let w = Weights::from_vec(&weights);
    Ok(evaluate_with_weights(&pos, &w))
}

#[pyfunction]
fn compute_mse(positions: Vec<(String, f32)>, weights: Vec<i32>, k: f32) -> PyResult<f32> {
    let w = Weights::from_vec(&weights);

    const SCALE: f64 = 1e9;
    
    // NOTE: Addition of floats is non-associative.
    // If using parallel iterator like Rayon, special care must be taken
    // to ensure compute_mse is deterministic.
    let total_error_int: i128 = positions.par_iter()
        .map(|(fen, result)| {
            let pos = Position::from_fen(fen);
            let score_side_to_move = evaluate_with_weights(&pos, &w) as f32;
            // Convert to white's perspective
            let score = if pos.is_white_turn { score_side_to_move } else { -score_side_to_move };
            let sigmoid = 1.0 / (1.0 + 10f32.powf(-k * score / 400.0));
            
            let error =(result - sigmoid).powi(2);

            (error as f64 * SCALE) as i128
        })
        .sum();
    
    let final_mse = (total_error_int as f64 / SCALE) / positions.len() as f64;
    
    Ok(final_mse as f32)
}

#[pymodule]
fn ratsu(m: &Bound<'_, PyModule>) -> PyResult<()> {
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
    pawn_pst: [i32; 64],
    knight_pst: [i32; 64],
    bishop_pst: [i32; 64],
    rook_pst: [i32; 64],
    queen_pst: [i32; 64],
    king_pst: [i32; 64],
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
    pub fn to_vec(&self) -> Vec<i32> {
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
        v.extend_from_slice(&self.pawn_pst);
        v.extend_from_slice(&self.knight_pst);
        v.extend_from_slice(&self.bishop_pst);
        v.extend_from_slice(&self.rook_pst);
        v.extend_from_slice(&self.queen_pst);
        v.extend_from_slice(&self.king_pst);
        v
    }

    pub fn from_vec(v: &[i32]) -> Self {
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
            pawn_pst: v[14..78].try_into().unwrap(),
            knight_pst: v[78..142].try_into().unwrap(),
            bishop_pst: v[142..206].try_into().unwrap(),
            rook_pst: v[206..270].try_into().unwrap(),
            queen_pst: v[270..334].try_into().unwrap(),
            king_pst: v[334..398].try_into().unwrap(),
        }
    }
}

const DEFAULT_WEIGHTS: Weights = Weights {
    material_pawn: 100,
    material_knight: 350,
    material_bishop: 350,
    material_rook: 525,
    material_queen: 1000,
    material_king: 20000,
    #[rustfmt::skip]
    pawn_pst: [
        0,  0,  0,  0,  0,  0,  0,  0,
        50, 50, 50, 50, 50, 50, 50, 50,
        10, 10, 20, 30, 30, 20, 10, 10,
        5,  5, 10, 25, 25, 10,  5,  5,
        0,  0,  0, 20, 20,  0,  0,  0,
        5, -5, -5,  0,  0,-10,  0,  5,
        5, 10, 10,-20,-20, 10, 10,  5,
        0,  0,  0,  0,  0,  0,  0,  0
    ],
    #[rustfmt::skip]
    knight_pst: [
        -50,-40,-30,-30,-30,-30,-40,-50,
        -40,-20,  0,  0,  0,  0,-20,-40,
        -30,  0, 10, 15, 15, 10,  0,-30,
        -20,  5, 15, 20, 20, 15,  5,-20,
        -20,  0, 15, 20, 20, 15,  0,-20,
        -30,  5, 10, 15, 15, 10,  5,-30,
        -40,-20,  0,  5,  5,  0,-20,-40,
        -50,-40,-30,-30,-30,-30,-40,-50,
    ],
    #[rustfmt::skip]
    bishop_pst: [
        -20,-10,-10,-10,-10,-10,-10,-20,
        -10,  0,  0,  0,  0,  0,  0,-10,
        -10,  0,  5, 10, 10,  5,  0,-10,
        -10,  5,  5, 10, 10,  5,  5,-10,
        -10,  0, 10, 10, 10, 10,  0,-10,
        -10, 10, 10, 10, 10, 10, 10,-10,
        -10, 10,  0,  0,  0,  0, 10,-10,
        -20,-10,-10,-10,-10,-10,-10,-20,
    ],
    #[rustfmt::skip]
    rook_pst: [
        5,  5,  5,  5,  5,  0,  0,  0,
        5, 10, 10, 10, 10, 10, 10,  5,
        -5,  0,  0,  0,  0,  0,  0, -5,
        -5,  0,  0,  0,  0,  0,  0, -5,
        -5,  0,  0,  0,  0,  0,  0, -5,
        -5,  0,  0,  0,  0,  0,  0, -5,
        -5,  0,  0,  0,  0,  0,  0, -5,
        0,  0,  0,  5,  5,  0,  0,  0,
    ],
    #[rustfmt::skip]
    queen_pst: [
        -20,-10, -5, -0, -0, -0, -5,-10,
        -10,  0,  0,  0,  0,  5,  5, -5,
        -10,  0,  5,  5,  5,  5,  5,-10,
        -5,  0,  5,  5,  5,  5,  0, -5,
        0,  0,  5,  5,  5,  5,  0, -5,
        -10,  5,  5,  5,  5,  5,  5,-10,
        -10,  0,  5,  0,  0,  0,  0,-10,
        -20,-10,-10, -5, -5,-10,-10,-20,
    ],
    #[rustfmt::skip]
    king_pst: [
        0,  0,  0,  0,   0,  0,  0,  0,
        0,  0,  5,  5,   5,  5,  0,  0,
        0,  5,  5, 10,  10,  5,  5,  0,
        0,  5, 10, 20,  20, 10,  5,  0,
        0,  5, 10, 20,  20, 10,  5,  0,
        0,  0,  5, 10,  10,  5,  0,  0,
        0,  5,  5, -5,  -5,  0,  5,  0,
        0,  0,  5,  0, -15,  0,  10, 0,
    ],
    doubled_pawn_penalty: 10,
    isolated_pawn_penalty: 20,
    backwards_pawn_penalty: 8,
    passed_pawn_bonus: 20,
    rook_semi_open_file_bonus: 10,
    rook_open_file_bonus: 15,
    rook_on_seventh_bonus: 20,
    bishop_pair_bonus: 35,
};

fn evaluate_with_weights(position: &Position, weights: &Weights) -> i32 {
    let mut score = 0;
    let side = if position.is_white_turn { 1 } else { -1 };
    let (white_pawn_ranks, black_pawn_ranks) = init_pawn_ranks(position);
    let mut white_bishops = 0;
    let mut black_bishops = 0;

    for rank in 0..8 {
        for file in 0..8 {
            let square = rank * 16 + file;
            let piece = position.board[square];
            let piece_type = get_piece_type(piece);
            if piece_type == EMPTY {
                continue;
            }

            score += get_piece_table_score_with_weights(square, piece, piece_type, weights);
            score += get_piece_material_score_with_weights(piece, weights);
            if piece_type == BISHOP {
                if get_piece_color(piece) == WHITE {
                    white_bishops += 1;
                } else {
                    black_bishops += 1;
                }
            }
            if piece_type == PAWN {
                score += get_pawn_structure_score_with_weights(
                    &white_pawn_ranks,
                    &black_pawn_ranks,
                    piece,
                    rank as u8,
                    file + 1,
                    weights,
                );
            }
            if piece_type == ROOK {
                score += get_rook_score_with_weights(
                    &white_pawn_ranks,
                    &black_pawn_ranks,
                    piece,
                    rank as u8,
                    file + 1,
                    weights,
                );
            }
        }
    }
    if white_bishops >= 2 {
        score += weights.bishop_pair_bonus;
    }
    if black_bishops >= 2 {
        score -= weights.bishop_pair_bonus;
    }
    score * side
}

fn get_piece_table_score_with_weights(
    square: usize,
    piece: u8,
    piece_type: u8,
    weights: &Weights,
) -> i32 {
    let square64 = get_square_in_64(square);

    if get_piece_color(piece) == WHITE {
        match piece_type {
            PAWN => weights.pawn_pst[square64],
            KNIGHT => weights.knight_pst[square64],
            BISHOP => weights.bishop_pst[square64],
            ROOK => weights.rook_pst[square64],
            QUEEN => weights.queen_pst[square64],
            KING => weights.king_pst[square64],
            _ => panic!("Unexpected piece {}", piece),
        }
    } else {
        match piece_type {
            PAWN => -flip_board(&weights.pawn_pst)[square64],
            KNIGHT => -flip_board(&weights.knight_pst)[square64],
            BISHOP => -flip_board(&weights.bishop_pst)[square64],
            ROOK => -flip_board(&weights.rook_pst)[square64],
            QUEEN => -flip_board(&weights.queen_pst)[square64],
            KING => -flip_board(&weights.king_pst)[square64],
            _ => panic!("Unexpected piece {}", piece),
        }
    }
}

pub fn get_material_score_with_weights(piece: u8, weights: &Weights) -> i32 {
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

fn get_pawn_structure_score_with_weights(
    white_pawn_ranks: &[u8; 10],
    black_pawn_ranks: &[u8; 10],
    piece: u8,
    rank: u8,
    pawn_file: usize,
    weights: &Weights,
) -> i32 {
    let mut score = 0;
    let left_file = pawn_file - 1;
    let right_file = pawn_file + 1;
    if get_piece_color(piece) == WHITE {
        if white_pawn_ranks[pawn_file] > rank {
            score -= weights.doubled_pawn_penalty;
        }

        if white_pawn_ranks[left_file] == 0 && white_pawn_ranks[right_file] == 0 {
            score -= weights.isolated_pawn_penalty;
        } else if rank > white_pawn_ranks[left_file] && rank > white_pawn_ranks[right_file] {
            score -= weights.backwards_pawn_penalty;
        }

        if rank <= black_pawn_ranks[left_file]
            && rank <= black_pawn_ranks[pawn_file]
            && rank <= black_pawn_ranks[right_file]
        {
            score += (7 - rank as i32) * weights.passed_pawn_bonus;
        }
    } else {
        if black_pawn_ranks[pawn_file] < rank {
            score += weights.doubled_pawn_penalty;
        }
        if black_pawn_ranks[left_file] == 7 && black_pawn_ranks[right_file] == 7 {
            score += weights.isolated_pawn_penalty;
        } else if rank < black_pawn_ranks[left_file] && rank < black_pawn_ranks[right_file] {
            score += weights.backwards_pawn_penalty;
        }

        if rank >= white_pawn_ranks[left_file]
            && rank >= white_pawn_ranks[pawn_file]
            && rank >= white_pawn_ranks[right_file]
        {
            score -= rank as i32 * weights.passed_pawn_bonus
        }
    }
    score
}

fn get_rook_score_with_weights(
    white_pawn_ranks: &[u8; 10],
    black_pawn_ranks: &[u8; 10],
    piece: u8,
    rank: u8,
    pawn_file: usize,
    weights: &Weights,
) -> i32 {
    let mut score = 0;
    if get_piece_color(piece) == WHITE {
        if black_pawn_ranks[pawn_file] == 7 {
            if white_pawn_ranks[pawn_file] == 0 {
                score += weights.rook_open_file_bonus
            } else {
                score += weights.rook_semi_open_file_bonus
            }
        }

        if rank == 1 {
            score += weights.rook_on_seventh_bonus
        }
    } else {
        if white_pawn_ranks[pawn_file] == 0 {
            if black_pawn_ranks[pawn_file] == 7 {
                score -= weights.rook_open_file_bonus
            } else {
                score -= weights.rook_semi_open_file_bonus
            }
        }

        if rank == 6 {
            score -= weights.rook_on_seventh_bonus
        }
    }
    score
}

#[cfg(test)]
mod tests {
    use crate::{START_POSITION_FEN, evaluation::evaluate};

    use super::*;

    #[test]
    fn test_evaluate_with_weights() {
        let pos = Position::from_fen(START_POSITION_FEN);
        let eval = evaluate(&pos);
        let eval_with_default_weights = evaluate_with_weights(&pos, &DEFAULT_WEIGHTS);
        assert_eq!(eval, eval_with_default_weights);

        let pawn_pos = Position::from_fen("4k3/1p2p3/4p1P1/3p4/3P4/4P2p/1P2P3/4K3 w - - 0 1");
        let pawn_eval = evaluate(&pawn_pos);
        let pawn_eval_with_default_weights = evaluate_with_weights(&pawn_pos, &DEFAULT_WEIGHTS);
        assert_eq!(pawn_eval, pawn_eval_with_default_weights);

        let rook_pos = Position::from_fen("2r1kr2/Rp1p1p2/8/8/8/8/rP1P2P1/2R1K1R1 w - - 0 1");
        let rook_eval = evaluate(&rook_pos);
        let rook_eval_with_default_weights = evaluate_with_weights(&rook_pos, &DEFAULT_WEIGHTS);
        assert_eq!(rook_eval, rook_eval_with_default_weights);
    }
    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_to_vec_length() {
            let w = DEFAULT_WEIGHTS;
            assert_eq!(w.to_vec().len(), 398); // 14 scalars + 6*64 PST values
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
            assert_eq!(v[14..78], w.pawn_pst);
            assert_eq!(v[78..142], w.knight_pst);
            assert_eq!(v[334..398], w.king_pst);
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
}
