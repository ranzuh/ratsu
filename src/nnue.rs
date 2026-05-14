use crate::{
    piece::{WHITE, get_piece_color, get_piece_type},
    position::Position,
};

const INPUT_SIZE: usize = 768;
pub const HIDDEN_SIZE: usize = 256;
const QA: i32 = 256;
const QB: i32 = 64;

pub struct Nnue {
    pub acc_weights: [[i16; HIDDEN_SIZE]; INPUT_SIZE],
    pub acc_bias: [i16; HIDDEN_SIZE],
    pub out_weights: [i16; HIDDEN_SIZE],
    pub out_bias: i32,
}

impl Nnue {
    /// Load NNUE weights from a flat f32 binary file, quantizing to i16/i32 integers
    /// for fast inference. Weights are transposed for cache-friendly accumulator updates.
    pub fn load(nnue_bytes: &[u8]) -> Self {
        let mut offset = 0;

        let mut acc_weights = [[0i16; HIDDEN_SIZE]; INPUT_SIZE];
        for i in 0..HIDDEN_SIZE {
            for j in 0..INPUT_SIZE {
                let f = f32::from_le_bytes([
                    nnue_bytes[offset],
                    nnue_bytes[offset + 1],
                    nnue_bytes[offset + 2],
                    nnue_bytes[offset + 3],
                ]);
                acc_weights[j][i] = (f * QA as f32).round() as i16; // transpose + quantize
                offset += 4;
            }
        }

        let mut acc_bias = [0i16; HIDDEN_SIZE];
        for i in 0..HIDDEN_SIZE {
            let f = f32::from_le_bytes([
                nnue_bytes[offset],
                nnue_bytes[offset + 1],
                nnue_bytes[offset + 2],
                nnue_bytes[offset + 3],
            ]);
            acc_bias[i] = (f * QA as f32).round() as i16;
            offset += 4;
        }

        let mut out_weights = [0i16; HIDDEN_SIZE];
        for i in 0..HIDDEN_SIZE {
            let f = f32::from_le_bytes([
                nnue_bytes[offset],
                nnue_bytes[offset + 1],
                nnue_bytes[offset + 2],
                nnue_bytes[offset + 3],
            ]);
            out_weights[i] = (f * QB as f32 * 200.0).round() as i16;
            offset += 4;
        }

        let out_bias_f = f32::from_le_bytes([
            nnue_bytes[offset],
            nnue_bytes[offset + 1],
            nnue_bytes[offset + 2],
            nnue_bytes[offset + 3],
        ]);
        let out_bias = (out_bias_f * QA as f32 * QB as f32 * 200.0).round() as i32;

        Nnue {
            acc_weights,
            acc_bias,
            out_weights,
            out_bias,
        }
    }

    pub fn forward_from_accumulator(&self, accumulator: &[i16; HIDDEN_SIZE]) -> i32 {
        let mut out: i32 = self.out_bias;
        for j in 0..HIDDEN_SIZE {
            let clamped = accumulator[j].clamp(0, QA as i16) as i32;
            out += (self.out_weights[j] as i32) * clamped;
        }
        out / (QA * QB)
    }
}

pub fn get_feature_index(sq128: usize, piece: u8) -> usize {
    let white_piece = get_piece_color(piece) == WHITE;
    let color = if white_piece { 0 } else { 1 };
    let pt = (get_piece_type(piece) - 1) as usize;
    let sq64 = (sq128 >> 4) * 8 + (sq128 & 7);
    color * 384 + pt * 64 + sq64
}

pub fn nnue_evaluate(position: &Position, nnue: &Nnue) -> i32 {
    let raw = nnue.forward_from_accumulator(&position.accumulator);
    let side = if position.is_white_turn { 1 } else { -1 };
    raw * side
}
