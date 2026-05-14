pub mod hash;
pub mod movegen;
pub mod moveordering;
pub mod nnue;
pub mod perft;
pub mod piece;
pub mod position;
pub mod search;
pub mod uci;
// pub mod bitboard;
// pub mod evaluation;

#[cfg(feature = "python")]
pub mod tuner;

pub const START_POSITION_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
