use rustchess::{
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

#[test]
fn perft_suite() {
    let test_positions = [
        START_POSITION,
        KIWIPETE,
        POSITION_3,
        POSITION_4,
        POSITION_5,
        POSITION_6,
    ];
    for test_pos in test_positions {
        let mut position = Position::from_fen(test_pos.fen_string);
        let mut counts = PerftCounts::default();
        let total_nodes = perft(test_pos.depth, &mut position, &mut counts, true);
        assert_eq!(total_nodes, test_pos.nodes)
    }
}
