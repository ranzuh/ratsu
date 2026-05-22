use crate::{
    piece::{WHITE, get_piece_color, get_piece_type},
    position::Position,
};

const INPUT_SIZE: usize = 768;
pub const ACC_SIZE: usize = 128;
const QA: i32 = 256;
const QB: i32 = 64;
const SCALE: f32 = 200.0;

pub struct Nnue {
    pub acc_weights: [[i16; ACC_SIZE]; INPUT_SIZE],
    pub acc_bias: [i16; ACC_SIZE],
    pub out_weights: [i16; ACC_SIZE * 2], // takes [stm, opp] concatenated
    pub out_bias: i32,
}

fn next_float_from_bytes(bytes: &[u8], offset: usize) -> f32 {
    f32::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ])
}

impl Nnue {
    /// Load NNUE weights from a flat f32 binary file, quantizing to i16/i32 integers
    /// for fast inference. Weights are transposed for cache-friendly accumulator updates.
    pub fn load(nnue_bytes: &[u8]) -> Self {
        let mut offset = 0;

        let mut acc_weights = [[0i16; ACC_SIZE]; INPUT_SIZE];
        for i in 0..ACC_SIZE {
            for j in 0..INPUT_SIZE {
                let f = next_float_from_bytes(nnue_bytes, offset);
                acc_weights[j][i] = (f * QA as f32).round() as i16;
                offset += 4;
            }
        }

        let mut acc_bias = [0i16; ACC_SIZE];
        for i in 0..ACC_SIZE {
            let f = next_float_from_bytes(nnue_bytes, offset);
            acc_bias[i] = (f * QA as f32).round() as i16;
            offset += 4;
        }

        let mut out_weights = [0i16; ACC_SIZE * 2];
        for i in 0..ACC_SIZE * 2 {
            let f = next_float_from_bytes(nnue_bytes, offset);
            out_weights[i] = (f * QB as f32 * SCALE).round() as i16;
            offset += 4;
        }

        let out_bias_f = next_float_from_bytes(nnue_bytes, offset);
        let out_bias = (out_bias_f * QA as f32 * QB as f32 * SCALE).round() as i32;

        Nnue {
            acc_weights,
            acc_bias,
            out_weights,
            out_bias,
        }
    }

    pub fn forward_from_accumulator(
        &self,
        white_acc: &[i16; ACC_SIZE],
        black_acc: &[i16; ACC_SIZE],
        is_white: bool,
    ) -> i32 {
        let (stm, opp) = if is_white {
            (white_acc, black_acc)
        } else {
            (black_acc, white_acc)
        };
        let mut out: i32 = self.out_bias;
        for j in 0..ACC_SIZE {
            let s = stm[j].clamp(0, QA as i16) as i32;
            let o = opp[j].clamp(0, QA as i16) as i32;
            out += self.out_weights[j] as i32 * s;
            out += self.out_weights[ACC_SIZE + j] as i32 * o;
        }
        out / (QA * QB)
    }
}

pub fn get_feature_index(sq128: usize, piece: u8, is_white_turn: bool) -> usize {
    let is_white_piece = get_piece_color(piece) == WHITE;
    let piece_type = (get_piece_type(piece) - 1) as usize;
    let row = sq128 >> 4;
    let col = sq128 & 7;

    if is_white_turn {
        let color = if is_white_piece { 0 } else { 1 };
        let sq64 = row * 8 + col;
        color * 384 + piece_type * 64 + sq64
    } else {
        // Flip: enemy/friendly swapped, rank mirrored
        let color = if is_white_piece { 1 } else { 0 };
        let sq64 = (7 - row) * 8 + col;
        color * 384 + piece_type * 64 + sq64
    }
}

pub fn nnue_evaluate(position: &Position, nnue: &Nnue) -> i32 {
    nnue.forward_from_accumulator(
        &position.acc_white,
        &position.acc_black,
        position.is_white_turn,
    )
}
