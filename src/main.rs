use ratsu::{
    START_POSITION_FEN,
    movegen::BOARD_SQUARES,
    nnue::Nnue,
    piece::{EMPTY, WHITE, get_piece_color, get_piece_type},
    position::Position,
    uci::uci_loop,
};

fn main() {
    // const NNUE_BYTES: &[u8] = include_bytes!("../nnue/nnue.bin");
    // let nnue = Nnue::load(NNUE_BYTES);
    // let pos = Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNB1KBNR w KQkq - 0 1");

    // let mut features = Vec::with_capacity(32);
    // for sq128 in BOARD_SQUARES {
    //     let piece = pos.board[sq128];
    //     if piece == EMPTY {
    //         continue;
    //     }
    //     let color = if get_piece_color(piece) == WHITE {
    //         0
    //     } else {
    //         1
    //     };
    //     let pt = (get_piece_type(piece) - 1) as usize;
    //     let sq64 = (sq128 >> 4) * 8 + (sq128 & 7);
    //     features.push(color * 384 + pt * 64 + sq64);
    // }

    // println!("{}", nnue.forward(&features));

    uci_loop();
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
