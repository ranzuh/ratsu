mod movegen;
mod perft;
mod piece;
mod position;

use movegen::{Move, generate_pseudo_moves, get_move_string, is_square_attacked};
use perft::{PerftCounts, run_perft};
use position::Position;

const START_POSITION_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

fn main() {
    // let custom_fen = "2n1k3/1P6/8/8/8/8/8/4K3 w - - 0 1";
    // let mut pos = Position::from_fen(custom_fen);
    // pos.print();
    // let moves = pos.generate_legal_moves();
    // for move_ in &moves {
    //     println!("{}", get_move_string(&move_));
    // }
    // pos.make_move(&moves[4]);
    // pos.print();
    // let move_ = Move {
    //     from: square,
    //     to: target_square,
    //     promoted_piece: None,
    //     is_capture: true,
    //     is_enpassant: true,
    //     is_double_pawn: false,
    //     is_castling: false,
    // };
    let custom_fen = "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8";
    let mut pos = Position::from_fen(custom_fen);
    run_perft(4, &mut pos);
}

// 8   0,   1,   2,   3,   4,   5,   6,   7,                   8, 9, 10, 11, 12, 13, 14, 15,
// 7  16,  17,  18,  19,  20,  21,  22,  23,           24, 25, 26, 27, 28, 29, 30, 31,
// 6  32,  33,  34,  35,  36,  37,  38,  39,           40, 41, 42, 43, 44, 45, 46, 47,
// 5  48,  49,  50,  51,  52,  53,  54,  55,           56, 57, 58, 59, 60, 61, 62, 63,
// 4  64,  65,  66,  67,  68,  69,  70,  71,           72, 73, 74, 75, 76, 77, 78, 79,
// 3  80,  81,  82,  83,  84,  85,  86,  87,           88, 89, 90, 91, 92, 93, 94, 95,
// 2  96,  97,  98,  99, 100, 101, 102, 103,       104, 105, 106, 107, 108, 109, 110, 111,
// 1 112, 113, 114, 115, 116, 117, 118, 119,   120, 121, 122, 123, 124, 125, 126, 127,
//   a    b    c    d    e    f    g    h
