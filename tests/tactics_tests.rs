use ratsu::{
    hash::TranspositionTable, movegen::get_move_string, position::Position, search::Search,
};

#[rustfmt::skip]
    const WAC_POSITIONS: &[(&str, &str, u32)] = &[
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
    ];

#[test]
fn win_at_chess() {
    let mut tt = TranspositionTable::new(64);
    for (fen, exp_move, depth) in WAC_POSITIONS {
        tt.clear();
        let mut pos = Position::from_fen(fen);
        println!("{}", fen);
        let movetime = 10000;
        let (pv, _node_count) = Search::run(&mut pos, &mut tt, *depth, movetime);
        let best_move = pv.get(0).expect("pv should have moves");
        assert_eq!(get_move_string(&best_move), *exp_move);
    }
}
