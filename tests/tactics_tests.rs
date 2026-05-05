use ratsu::{
    hash::TranspositionTable, movegen::get_move_string, position::Position, search::Search,
};

#[rustfmt::skip]
    const TACTICS: &[(&str, &str, u32)] = &[
        // WIN AT CHESS POSITIONS
        ("2rr3k/pp3pp1/1nnqbN1p/3pN3/2pP4/2P3Q1/PPB4P/R4RK1 w - -", "g3g6", 4),
        ("5rk1/1ppb3p/p1pb4/6q1/3P1p1r/2P1R2P/PP1BQ1P1/5RKN w - -", "e3g3", 4),
        ("r1bq2rk/pp3pbp/2p1p1pQ/7P/3P4/2PB1N2/PP3PPR/2KR4 w - -", "h6h7", 4),
        ("5k2/6pp/p1qN4/1p1p4/3P4/2PKP2Q/PP3r2/3R4 b - -", "c6c4", 4),
        ("rnbqkb1r/pppp1ppp/8/4P3/6n1/7P/PPPNPPP1/R1BQKBNR b KQkq -", "g4e3", 6),
        ("2br2k1/2q3rn/p2NppQ1/2p1P3/Pp5R/4P3/1P3PPP/3R2K1 w - -", "h4h7", 4),
        ("r1b1kb1r/3q1ppp/pBp1pn2/8/Np3P2/5B2/PPP3PP/R2Q1RK1 w kq -", "f3c6", 5),
        ("4k1r1/2p3r1/1pR1p3/3pP2p/3P2qP/P4N2/1PQ4P/5R1K b - -", "g4f3", 4),
        ("5rk1/pp4p1/2n1p2p/2Npq3/2p5/6P1/P3P1BP/R4Q1K w - -", "f1f8", 4),
        ("r2rb1k1/pp1q1p1p/2n1p1p1/2bp4/5P2/PP1BPR1Q/1BPN2PP/R5K1 w - -", "h3h7", 6),
        ("1R6/1brk2p1/4p2p/p1P1Pp2/P7/6P1/1P4P1/2R3K1 w - -", "b8b7", 5),
        ("r4rk1/ppp2ppp/2n5/2bqp3/8/P2PB3/1PP1NPPP/R2Q1RK1 w - -", "e2c3", 5),
        ("r1b2rk1/ppbn1ppp/4p3/1QP4q/3P4/N4N2/5PPP/R1B2RK1 w - -", "c5c6", 5),
        ("5rk1/1b3p1p/pp3p2/3n1N2/1P6/P1qB1PP1/3Q3P/4R1K1 w - -", "d2h6", 6),
        // 60 3-move mate puzzles from Classic Chess Puzzles by Chessnut1071
        ("8/1n3Np1/1N4Q1/1bkP4/p1p2p2/P1P2R2/3P2PK/B2R4 w - -", "d1h1", 6),
        ("6R1/2K5/8/8/5p2/6p1/1Q3Nk1/8 w - -", "f2g4", 6),
        ("6b1/b2nB3/4p2r/1Np1kp2/2p2N2/K1P1PP2/2B5/8 w - -", "c2a4", 6),
        // ("8/2P1Q2b/2qp3n/5k1B/5N1R/4K3/7B/8 w - -", "e3d2", 9),
        ("5N1k/5Ppp/8/8/2Q3p1/8/8/b6K w - -", "c4f1", 6),
        ("4r1b1/1p4B1/pN2pR2/RB2k3/1P2N2p/2p3b1/n2P1p1r/5K1n w - -", "f1e2", 6),
        ("K5R1/1R5p/1P5k/3b2Np/3B1P1b/6pP/6P1/8 w - -", "f4f5", 6),
        ("4r1nb/2B2R2/1p2k2p/1P2pR2/1rb1p2K/4P1pB/3n1N1P/4q3 w - -", "f2d3", 6),
        ("8/P3P3/p7/Pk6/4N3/1K6/8/8 w - -", "a7a8q", 6),
        ("8/4p2R/3PkP2/R7/8/8/8/4K3 w - -", "f6e7", 6),
        ("8/4K3/2k5/5Q1B/bp6/8/7B/8 w - -", "h5e2", 6),
        ("2R5/3K2k1/8/p7/3P4/8/6p1/1Q6 w - -", "c8c2", 6),
        ("8/8/8/8/nnN5/Q7/8/1kN4K w - -", "c1b3", 6),
        ("7b/8/r7/3B4/8/8/8/1kBK2Q1 w - -", "d1e2", 6),
        ("8/5p2/5Q2/8/8/5R2/4K3/Bkb5 w - -", "f3f5", 6),
        ("1q5k/4R3/8/8/1p6/1B6/5R2/1K6 w - -", "f2a2", 6),
        ("8/8/8/8/1N3Rp1/8/Bk1B2K1/8 w - -", "a2b1", 6),
        ("8/1b6/3Q4/8/4N3/8/1K4p1/4N2k w - -", "d6d1", 6),
        ("4Q3/8/2K5/8/8/N1R5/1p3p2/k7 w - -", "a3c4", 6),
        ("N7/kP1R4/1N6/2K5/8/8/8/8 w - -", "a8c7", 6),
        ("3n1Q2/6p1/1n6/1P2P1b1/3Pk3/2P3pP/BP4N1/4K3 w - -", "f8f1", 6),
        ("8/P2B1r2/3p1p2/3n4/1P1k1Pp1/4p3/1K2Q2b/6N1 w - -", "a7a8q", 6),
        ("8/8/2K4n/p1N1k2p/2p3P1/7n/bR1P2N1/5Q2 w - -", "b2b4", 6),
        ("r1NR2br/7p/p2Pkp2/1pQ5/5K2/8/8/8 w - -", "d6d7", 6),
        ("8/7p/2p1N2P/2Pk2K1/4pPB1/1P2R3/5B2/3n4 w - -", "g5f5", 6),
        ("R2B4/3k2N1/6Pn/nK2P3/2p2N2/8/1p4r1/1b2bQ2 w - -", "f1f2", 6),
        ("8/8/4p3/QK1k4/8/3B4/5N2/n7 w - -", "d3h7", 6),
        ("8/4N3/1p6/6K1/2nk4/6Q1/1n2R3/7B w - -", "e2c2", 6),
        ("5K2/5p2/1p1N4/b2k4/Bp2R1p1/1N4B1/3rP3/3n4 w - -", "a4d7", 6),
        ("K5b1/8/Q7/3kB3/4p1N1/1p1r3p/1P2N1Pr/5n2 w - -", "e5b8", 6),
        ("b5Kn/p5Bp/4P1p1/1P1k4/p1pNN2P/5BP1/8/7Q w - -", "d4f5", 6),
        ("3nR1K1/nQp3N1/5kp1/6p1/2r3p1/3r1N2/8/8 w - -", "f3e5", 6),
        ("8/6p1/1Nr5/pr6/3p2PB/1PpBk2p/2P5/5R1K w - -", "h4e7", 6),
        ("1N1q1r1n/4B3/pp4rp/4k3/2Q3RN/4p1pK/8/1b1Bb3 w - -", "d1b3", 6),
        ("3bq3/N2r4/pp3rb1/pR1p4/3k1P2/1PpN2B1/2P1PP2/6KB w - -", "g3h4", 6),
        ("K7/1B6/Nk6/pb6/2P3p1/8/Q7/b5q1 w - -", "a2d2", 6),
        ("8/K1p2N2/8/1p1rp2b/nPrkP2R/2N1p2q/1PQP4/4B3 w - -", "f7d8", 6),
        ("1rn3b1/7p/Q6N/2R1B3/4kp2/pq1p4/1pp2K2/8 w - -", "e5c3", 6),
        ("1B1N1r1Q/4p3/bn1p2r1/R2Bpp2/q2k4/2R3Np/3P3K/8 w - -", "d5g2", 6),
        ("b2r1N1n/8/r4p1p/4pp1R/1pP2k1K/5pN1/p2p1P2/1q1Q3B w - -", "c4c5", 6),
        ("6rB/4p2r/2B1pnp1/2K1k3/4NNP1/3pPp2/3p1p2/3bbq2 w - -", "e4g5", 6),
        ("8/6P1/8/1K6/2R5/8/1k6/8 w - -", "g7g8r", 8),
        ("8/8/8/8/8/6p1/2RR4/r3k1K1 w - -", "d2g2", 6),
        ("3R3N/2pR1p2/4k3/N1P5/1q4PK/2B4B/8/8 w - -", "c5c6", 6),
        ("2B5/2p1pK2/Q2pN3/3Rbk1p/2P2pb1/7r/4P3/4BN2 w - -", "a6a3", 6),
        ("7K/8/3p3N/4kBB1/1N3pp1/2P5/5PPP/8 w - -", "h2h4", 6),
        ("B4n2/3ppp2/1Np1k2N/2P1P3/2p1p2p/6B1/3P1P2/Q3R1K1 w - -", "d2d4", 6),
        ("8/p4p2/2P2P1R/R1K1kN1P/2p5/2r1N1Pn/BB1P1P2/3nQ1b1 w - -", "f2f3", 6),
        ("b7/PP6/8/8/7K/6B1/6N1/4R1bk w - -", "b7a8n", 6),
        ("4K3/5P2/2N1k3/8/4P3/3R4/6P1/8 w - -", "f7f8b", 6),
        ("2K5/R7/3Pk3/4p3/3PPB2/3P1P2/8/8 w - -", "d6d7", 6),
        ("8/8/1Q3K2/3pN3/4kp2/8/8/8 w - -", "b6g1", 6),
        ("6K1/8/7p/5p2/4k3/1Q3N2/2P3R1/7n w - -", "b3c3", 6),
        ("k4K2/3R4/8/3N4/1R6/5b2/8/8 w - -", "f8g7", 6),
        ("k6K/3R4/8/1R1N3p/8/8/6b1/8 w - -", "b5b4", 6)
    ];

#[test]
fn win_at_chess() {
    let mut tt = TranspositionTable::new(64);
    for (fen, exp_move, depth) in TACTICS {
        tt.clear();
        let mut pos = Position::from_fen(fen);
        println!("{}", fen);
        let movetime = 10000;
        let (pv, _node_count) = Search::run(&mut pos, &mut tt, *depth, movetime, false);
        let best_move = pv.get(0).expect("pv should have moves");
        assert_eq!(get_move_string(&best_move), *exp_move);
    }
}
