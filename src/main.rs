mod movegen;
mod perft;
mod piece;
mod position;

use movegen::{Move, generate_pseudo_moves, is_square_attacked, print_move};
use perft::{CAPTURES, CASTLINGS, ENPASSANTS, PROMOTIONS, perft};
use position::Position;

const START_POSITION_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

fn main() {
    let custom_fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - ";
    let mut pos = Position::from_fen(START_POSITION_FEN);
    pos.print();
    // let move_ = Move {
    //     from: 49,
    //     to: 33,
    //     promoted_piece: None,
    //     is_capture: false,
    //     is_enpassant: false,
    //     is_double_pawn: false,
    //     is_castling: false,
    // };
    // pos.make_move(&move_);
    // pos.print();
    // let moves = pos.generate_legal_moves();
    // for move_ in moves {
    //     print_move(&move_);
    // }
    #[allow(static_mut_refs)]
    unsafe {
        println!("{}", perft(5, &mut pos));
        println!("captures: {}", CAPTURES);
        println!("castles: {}", CASTLINGS);
        println!("enpassants: {}", ENPASSANTS);
        println!("promotions: {}", PROMOTIONS);
    }
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
