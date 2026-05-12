use crate::{
    movegen::BOARD_SQUARES,
    piece::{EMPTY, WHITE, get_piece_color, get_piece_type},
    position::Position,
};

const INPUT_SIZE: usize = 768;
pub const H1_SIZE: usize = 512;
const H2_SIZE: usize = 32;

const QA: i32 = 256;

pub struct Nnue {
    pub w1: [[i16; INPUT_SIZE]; H1_SIZE], // 512 x 768
    pub b1: [i16; H1_SIZE],               // 512
    pub w2: [[i16; H1_SIZE]; H2_SIZE],    // 32 x 512
    pub b2: [i32; H2_SIZE],               // 32 (scaled by QA²)
    pub w3: [f32; H2_SIZE],               // 32
    pub b3: f32,                          // 1
}

impl Nnue {
    pub fn load(data: &[u8]) -> Self {
        let mut offset = 0;

        let mut w1 = [[0i16; INPUT_SIZE]; H1_SIZE];
        for r in 0..H1_SIZE {
            for c in 0..INPUT_SIZE {
                w1[r][c] = i16::from_le_bytes(data[offset..offset + 2].try_into().unwrap());
                offset += 2;
            }
        }

        let mut b1 = [0i16; H1_SIZE];
        for r in 0..H1_SIZE {
            b1[r] = i16::from_le_bytes(data[offset..offset + 2].try_into().unwrap());
            offset += 2;
        }

        let mut w2 = [[0i16; H1_SIZE]; H2_SIZE];
        for r in 0..H2_SIZE {
            for c in 0..H1_SIZE {
                w2[r][c] = i16::from_le_bytes(data[offset..offset + 2].try_into().unwrap());
                offset += 2;
            }
        }

        let mut b2 = [0i32; H2_SIZE];
        for r in 0..H2_SIZE {
            b2[r] = i32::from_le_bytes(data[offset..offset + 4].try_into().unwrap());
            offset += 4;
        }

        let mut w3 = [0f32; H2_SIZE];
        for r in 0..H2_SIZE {
            w3[r] = f32::from_le_bytes(data[offset..offset + 4].try_into().unwrap());
            offset += 4;
        }

        let b3 = f32::from_le_bytes(data[offset..offset + 4].try_into().unwrap());

        Nnue {
            w1,
            b1,
            w2,
            b2,
            w3,
            b3,
        }
    }

    // pub fn forward(&self, features: &[usize]) -> f32 {
    //     let mut h1 = self.b1;
    //     for &f in features {
    //         for j in 0..H1_SIZE {
    //             h1[j] += self.w1[j][f];
    //         }
    //     }
    //     for j in 0..H1_SIZE {
    //         h1[j] = h1[j].clamp(0.0, 1.0);
    //     }

    //     let mut h2 = self.b2;
    //     for j in 0..H2_SIZE {
    //         for k in 0..H1_SIZE {
    //             h2[j] += self.w2[j][k] * h1[k];
    //         }
    //     }
    //     for j in 0..H2_SIZE {
    //         h2[j] = h2[j].clamp(0.0, 1.0);
    //     }

    //     let mut out = self.b3;
    //     for j in 0..H2_SIZE {
    //         out += self.w3[j] * h2[j];
    //     }
    //     out
    // }

    pub fn forward_from_accumulator(&self, accumulator: &[i16; H1_SIZE]) -> f32 {
        // Clipped ReLU: clamp to [0, QA] in i16
        let mut h1 = [0i16; H1_SIZE];
        for j in 0..H1_SIZE {
            h1[j] = accumulator[j].clamp(0, QA as i16);
        }

        // L2: i16 × i16 → i32
        let mut h2 = self.b2; // already scaled by QA²
        for j in 0..H2_SIZE {
            for k in 0..H1_SIZE {
                h2[j] += self.w2[j][k] as i32 * h1[k] as i32;
            }
        }

        // Clipped ReLU on h2: values are at scale QA², clamp to [0, QA²]
        let qa_sq = (QA * QA) as i32;
        for j in 0..H2_SIZE {
            h2[j] = h2[j].clamp(0, qa_sq);
        }

        // L3: convert to float, divide by QA² to get back to real scale
        let mut out = self.b3;
        for j in 0..H2_SIZE {
            out += self.w3[j] * (h2[j] as f32 / qa_sq as f32);
        }
        out
    }
}

pub fn get_feature_index(sq128: usize, piece: u8) -> usize {
    let color = if get_piece_color(piece) == WHITE {
        0
    } else {
        1
    };
    let pt = (get_piece_type(piece) - 1) as usize;
    let sq64 = (sq128 >> 4) * 8 + (sq128 & 7);
    color * 384 + pt * 64 + sq64
}

// pub fn nnue_evaluate(position: &Position, nnue: &Nnue) -> i32 {
//     let mut features = Vec::with_capacity(32);
//     for sq128 in BOARD_SQUARES {
//         let piece = position.board[sq128];
//         if piece == EMPTY {
//             continue;
//         }
//         features.push(get_feature_index(sq128, piece));
//     }
//     let raw = nnue.forward(&features);
//     let scale = 400.0;
//     let score = (raw * scale) as i32;
//     let side = if position.is_white_turn { 1 } else { -1 };
//     score * side
// }

pub fn nnue_evaluate(position: &Position, nnue: &Nnue) -> i32 {
    let raw = nnue.forward_from_accumulator(&position.accumulator);
    let scale = 400.0;
    let score = (raw * scale) as i32;
    let side = if position.is_white_turn { 1 } else { -1 };
    score * side
}
