use ratsu::{
    perft::{PerftCounts, perft},
    position::Position,
};

struct PerftPosition {
    fen_string: &'static str,
    depth: u32,
    nodes: u64,
}

const START_POSITION: PerftPosition = PerftPosition {
    fen_string: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
    depth: 4,
    nodes: 197281,
};
const KIWIPETE: PerftPosition = PerftPosition {
    fen_string: "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -",
    depth: 3,
    nodes: 97862,
};
const POSITION_3: PerftPosition = PerftPosition {
    fen_string: "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
    depth: 5,
    nodes: 674624,
};
const POSITION_4: PerftPosition = PerftPosition {
    fen_string: "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
    depth: 4,
    nodes: 422333,
};
const POSITION_5: PerftPosition = PerftPosition {
    fen_string: "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
    depth: 3,
    nodes: 62379,
};
const POSITION_6: PerftPosition = PerftPosition {
    fen_string: "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
    depth: 3,
    nodes: 89890,
};

// Martin Sedlak's test positions
// Avoid illegal en passant
const AVOID_ILLEGAL_EP1: PerftPosition = PerftPosition {
    fen_string: "3k4/3p4/8/K1P4r/8/8/8/8 b - - 0 1",
    depth: 6,
    nodes: 1134888,
};
const AVOID_ILLEGAL_EP2: PerftPosition = PerftPosition {
    fen_string: "8/8/8/8/k1p4R/8/3P4/3K4 w - - 0 1",
    depth: 6,
    nodes: 1134888,
};
const AVOID_ILLEGAL_EP3: PerftPosition = PerftPosition {
    fen_string: "8/8/4k3/8/2p5/8/B2P2K1/8 w - - 0 1",
    depth: 6,
    nodes: 1015133,
};
const AVOID_ILLEGAL_EP4: PerftPosition = PerftPosition {
    fen_string: "8/b2p2k1/8/2P5/8/4K3/8/8 b - - 0 1",
    depth: 6,
    nodes: 1015133,
};

// En passant capture checks opponent
const EP_CAPTURE_CHECK1: PerftPosition = PerftPosition {
    fen_string: "8/8/1k6/2b5/2pP4/8/5K2/8 b - d3 0 1",
    depth: 6,
    nodes: 1440467,
};
const EP_CAPTURE_CHECK2: PerftPosition = PerftPosition {
    fen_string: "8/5k2/8/2Pp4/2B5/1K6/8/8 w - d6 0 1",
    depth: 6,
    nodes: 1440467,
};

// Short castling gives check
const SHORT_CASTLE_CHECK1: PerftPosition = PerftPosition {
    fen_string: "5k2/8/8/8/8/8/8/4K2R w K - 0 1",
    depth: 6,
    nodes: 661072,
};
const SHORT_CASTLE_CHECK2: PerftPosition = PerftPosition {
    fen_string: "4k2r/8/8/8/8/8/8/5K2 b k - 0 1",
    depth: 6,
    nodes: 661072,
};

// Long castling gives check
const LONG_CASTLE_CHECK1: PerftPosition = PerftPosition {
    fen_string: "3k4/8/8/8/8/8/8/R3K3 w Q - 0 1",
    depth: 6,
    nodes: 803711,
};
const LONG_CASTLE_CHECK2: PerftPosition = PerftPosition {
    fen_string: "r3k3/8/8/8/8/8/8/3K4 b q - 0 1",
    depth: 6,
    nodes: 803711,
};

// Castling (including losing castling rights due to rook capture)
const CASTLE_RIGHTS1: PerftPosition = PerftPosition {
    fen_string: "r3k2r/1b4bq/8/8/8/8/7B/R3K2R w KQkq - 0 1",
    depth: 4,
    nodes: 1274206,
};
const CASTLE_RIGHTS2: PerftPosition = PerftPosition {
    fen_string: "r3k2r/7b/8/8/8/8/1B4BQ/R3K2R b KQkq - 0 1",
    depth: 4,
    nodes: 1274206,
};

// Castling prevented
const CASTLE_PREVENTED1: PerftPosition = PerftPosition {
    fen_string: "r3k2r/8/3Q4/8/8/5q2/8/R3K2R b KQkq - 0 1",
    depth: 4,
    nodes: 1720476,
};
const CASTLE_PREVENTED2: PerftPosition = PerftPosition {
    fen_string: "r3k2r/8/5Q2/8/8/3q4/8/R3K2R w KQkq - 0 1",
    depth: 4,
    nodes: 1720476,
};

// Promote out of check
const PROMOTE_OUT_CHECK1: PerftPosition = PerftPosition {
    fen_string: "2K2r2/4P3/8/8/8/8/8/3k4 w - - 0 1",
    depth: 6,
    nodes: 3821001,
};
const PROMOTE_OUT_CHECK2: PerftPosition = PerftPosition {
    fen_string: "3K4/8/8/8/8/8/4p3/2k2R2 b - - 0 1",
    depth: 6,
    nodes: 3821001,
};

// Discovered check
const DISCOVERED_CHECK1: PerftPosition = PerftPosition {
    fen_string: "8/8/1P2K3/8/2n5/1q6/8/5k2 b - - 0 1",
    depth: 5,
    nodes: 1004658,
};
const DISCOVERED_CHECK2: PerftPosition = PerftPosition {
    fen_string: "5K2/8/1Q6/2N5/8/1p2k3/8/8 w - - 0 1",
    depth: 5,
    nodes: 1004658,
};

// Promote to give check
const PROMOTE_CHECK1: PerftPosition = PerftPosition {
    fen_string: "4k3/1P6/8/8/8/8/K7/8 w - - 0 1",
    depth: 6,
    nodes: 217342,
};
const PROMOTE_CHECK2: PerftPosition = PerftPosition {
    fen_string: "8/k7/8/8/8/8/1p6/4K3 b - - 0 1",
    depth: 6,
    nodes: 217342,
};

// Underpromotion to check
const UNDERPROMO_CHECK1: PerftPosition = PerftPosition {
    fen_string: "8/P1k5/K7/8/8/8/8/8 w - - 0 1",
    depth: 6,
    nodes: 92683,
};
const UNDERPROMO_CHECK2: PerftPosition = PerftPosition {
    fen_string: "8/8/8/8/8/k7/p1K5/8 b - - 0 1",
    depth: 6,
    nodes: 92683,
};

// Self stalemate
const SELF_STALEMATE1: PerftPosition = PerftPosition {
    fen_string: "K1k5/8/P7/8/8/8/8/8 w - - 0 1",
    depth: 6,
    nodes: 2217,
};
const SELF_STALEMATE2: PerftPosition = PerftPosition {
    fen_string: "8/8/8/8/8/p7/8/k1K5 b - - 0 1",
    depth: 6,
    nodes: 2217,
};

// Stalemate/checkmate
const STALEMATE_CHECKMATE1: PerftPosition = PerftPosition {
    fen_string: "8/k1P5/8/1K6/8/8/8/8 w - - 0 1",
    depth: 7,
    nodes: 567584,
};
const STALEMATE_CHECKMATE2: PerftPosition = PerftPosition {
    fen_string: "8/8/8/8/1k6/8/K1p5/8 b - - 0 1",
    depth: 7,
    nodes: 567584,
};

// Double check
const DOUBLE_CHECK1: PerftPosition = PerftPosition {
    fen_string: "8/8/2k5/5q2/5n2/8/5K2/8 b - - 0 1",
    depth: 4,
    nodes: 23527,
};
const DOUBLE_CHECK2: PerftPosition = PerftPosition {
    fen_string: "8/5k2/8/5N2/5Q2/2K5/8/8 w - - 0 1",
    depth: 4,
    nodes: 23527,
};

// Short castling impossible although the rook never moved away from its corner
const SHORT_CASTLE_IMPOSSIBLE1: PerftPosition = PerftPosition {
    fen_string: "1k6/1b6/8/8/7R/8/8/4K2R b K - 0 1",
    depth: 5,
    nodes: 1063513,
};
const SHORT_CASTLE_IMPOSSIBLE2: PerftPosition = PerftPosition {
    fen_string: "4k2r/8/8/7r/8/8/1B6/1K6 w k - 0 1",
    depth: 5,
    nodes: 1063513,
};

// Long castling impossible although the rook never moved away from its corner
const LONG_CASTLE_IMPOSSIBLE1: PerftPosition = PerftPosition {
    fen_string: "1k6/8/8/8/R7/1n6/8/R3K3 b Q - 0 1",
    depth: 5,
    nodes: 346695,
};
const LONG_CASTLE_IMPOSSIBLE2: PerftPosition = PerftPosition {
    fen_string: "r3k3/8/1N6/r7/8/8/8/1K6 w q - 0 1",
    depth: 5,
    nodes: 346695,
};

#[test]
fn perft_suite() {
    let test_positions = [
        START_POSITION,
        KIWIPETE,
        POSITION_3,
        POSITION_4,
        POSITION_5,
        POSITION_6,
        AVOID_ILLEGAL_EP1,
        AVOID_ILLEGAL_EP2,
        AVOID_ILLEGAL_EP3,
        AVOID_ILLEGAL_EP4,
        EP_CAPTURE_CHECK1,
        EP_CAPTURE_CHECK2,
        SHORT_CASTLE_CHECK1,
        SHORT_CASTLE_CHECK2,
        LONG_CASTLE_CHECK1,
        LONG_CASTLE_CHECK2,
        CASTLE_RIGHTS1,
        CASTLE_RIGHTS2,
        CASTLE_PREVENTED1,
        CASTLE_PREVENTED2,
        PROMOTE_OUT_CHECK1,
        PROMOTE_OUT_CHECK2,
        DISCOVERED_CHECK1,
        DISCOVERED_CHECK2,
        PROMOTE_CHECK1,
        PROMOTE_CHECK2,
        UNDERPROMO_CHECK1,
        UNDERPROMO_CHECK2,
        SELF_STALEMATE1,
        SELF_STALEMATE2,
        STALEMATE_CHECKMATE1,
        STALEMATE_CHECKMATE2,
        DOUBLE_CHECK1,
        DOUBLE_CHECK2,
        SHORT_CASTLE_IMPOSSIBLE1,
        SHORT_CASTLE_IMPOSSIBLE2,
        LONG_CASTLE_IMPOSSIBLE1,
        LONG_CASTLE_IMPOSSIBLE2,
    ];
    for test_pos in test_positions {
        let mut position = Position::from_fen(test_pos.fen_string);
        let mut counts = PerftCounts::default();
        let total_nodes = perft(test_pos.depth, &mut position, &mut counts, true);
        println!();
        assert_eq!(total_nodes, test_pos.nodes, "{}", test_pos.fen_string)
    }
}
