mod movegen;
mod perft;
mod piece;
mod position;

use movegen::{Move, generate_moves, print_move};
use perft::perft;
use position::Position;

const START_POSITION_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

fn main() {
    let custom_fen = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1 ";
    let mut pos = Position::from_fen(custom_fen);
    pos.print();
    // let move_ = Move {
    //     from: 0,
    //     to: 112,
    //     promoted_piece: None,
    //     is_capture: false,
    //     is_enpassant: false,
    //     is_double_pawn: false,
    //     is_castling: false,
    // };
    // pos.make_move(&move_);
    // pos.print();
    let moves = generate_moves(&pos);
    for move_ in moves {
        print_move(&move_);
    }
    println!("{}", perft(1, &mut pos));
}

// 0, 1, 2, 3, 4, 5, 6, 7,                   8, 9, 10, 11, 12, 13, 14, 15,
// 16, 17, 18, 19, 20, 21, 22, 23,           24, 25, 26, 27, 28, 29, 30, 31,
// 32, 33, 34, 35, 36, 37, 38, 39,           40, 41, 42, 43, 44, 45, 46, 47,
// 48, 49, 50, 51, 52, 53, 54, 55,           56, 57, 58, 59, 60, 61, 62, 63,
// 64, 65, 66, 67, 68, 69, 70, 71,           72, 73, 74, 75, 76, 77, 78, 79,
// 80, 81, 82, 83, 84, 85, 86, 87,           88, 89, 90, 91, 92, 93, 94, 95,
// 96, 97, 98, 99, 100, 101, 102, 103,       104, 105, 106, 107, 108, 109, 110, 111,
// 112, 113, 114, 115, 116, 117, 118, 119,   120, 121, 122, 123, 124, 125, 126, 127,
