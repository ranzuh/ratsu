use ratsu::{START_POSITION_FEN, position::Position, uci::handle_position};

#[test]
fn test_incremental_hash_changes() {
    // Test cases: (starting FEN, moves, expected final FEN)
    let test_cases = [
        (
            START_POSITION_FEN,
            "d2d4 g8f6 b1d2 d7d5 b2b4 c8f5 b4b5 c7c5",
            "rn1qkb1r/pp2pppp/5n2/1Ppp1b2/3P4/8/P1PNPPPP/R1BQKBNR w KQkq c6 0 5",
        ),
        (
            "rn1qkb1r/pp2pppp/5n2/1Ppp1b2/3P4/8/P1PNPPPP/R1BQKBNR w KQkq c6 0 5",
            "b5c6 e7e6 c1a3 f8a3 a1b1 e8g8",
            "rn1q1rk1/pp3ppp/2P1pn2/3p1b2/3P4/b7/P1PNPPPP/1R1QKBNR w K - 2 8",
        ),
        (
            START_POSITION_FEN,
            "e2e4 d7d5 e4d5 d8d5 b1c3 d5e6 g1e2 e6f5 d2d4 e7e6 c1f4 f8d6 f4d6 c7d6 e2g3 f5f4 g3h5 f4h6 c3b5 e8d7 d1f3 g8f6 h5f6 h6f6 f3g3 e6e5 e1c1 f6f4 g3f4 e5f4 f1c4 d7c6 d4d5 c6d7 c4e2 h8f8 d1d4 a7a6 b5c3 d7c7 d4f4 c8d7 h1e1 f7f5 e2d3 c7d8 c3e2 g7g6 f4b4 b7b5 e2d4 d8c8 e1e7 h7h5 e7g7 f8f6 b4b3 c8d8 b3c3 d8e8 d4c6 b8c6 d5c6 d7e6 c6c7 e8f8 g7h7 a8c8 a2a4 f8g8 h7e7 g8f8 e7e6 f6e6 c3c6 e6e1 c1d2 e1g1 g2g3 g1g2 a4b5 a6b5 d3b5 f8e7 c6c3 g2f2 d2e3 f2h2 b5a6 e7d7 a6c8 d7c8 e3f4 h5h4 g3h4 h2h4 f4g5 h4g4 g5f6 g4g2 f6e6 g2d2 b2b4 d2d4 b4b5 d4b4 e6d6 b4b5 c3b3 b5b3 c2b3 f5f4 b3b4 f4f3 b4b5 f3f2 b5b6 f2f1q d6c5 f1b1 c5c6 b1b4 c6d5 b4b6 d5e4 c8c7 e4e5 b6a5 e5d4 c7d6 d4e3 d6d5 e3f4 a5d2 f4g4 d2e3 g4h4 e3f4 h4h3 f4f3 h3h2 g6g5 h2g1 g5g4 g1h2 f3f2 h2h1 f2a2 h1g1 g4g3 g1h1 a2h2",
            "8/8/8/3k4/8/6p1/7q/7K w - - 2 77",
        ),
        (
            "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1",
            "h1h8 e8e7 a1b1",
            "r6R/4k3/8/8/8/8/8/1R2K3 b - - 2 2",
        ),
        (
            "1n5k/PPPPP3/8/8/8/8/ppppp3/1N5K w - - 0 1",
            "a7a8q c2b1b c7b8n",
            "QN5k/1P1PP3/8/8/8/8/pp1pp3/1b5K b - - 0 2",
        ),
        (
            "2k5/6p1/8/5P2/3p4/8/4P3/2K5 w - - 0 1",
            "e2e4 d4e3 c1d1 g7g5 f5g6",
            "2k5/8/6P1/8/8/4p3/8/3K4 b - - 0 3",
        ),
        (
            "r3k2r/p1pppppp/8/8/8/5B2/PPPPPPPP/R3K2R w KQkq - 0 1",
            "e1c1 e8g8 f3a8",
            "B4rk1/p1pppppp/8/8/8/8/PPPPPPPP/2KR3R b - - 0 2",
        ),
    ];

    for (start_fen, moves, expected_fen) in test_cases {
        // Build position incrementally by making moves
        let mut incremental_pos = Position::from_fen(start_fen);
        let position_command = format!("position fen {} moves {}", start_fen, moves);
        handle_position(&position_command, &mut incremental_pos);

        // Build position directly from final FEN
        let direct_pos = Position::from_fen(expected_fen);

        // Verify that incremental hash matches direct hash
        assert_eq!(
            incremental_pos.hash, direct_pos.hash,
            "Hash mismatch for moves: {} (starting from: {})",
            moves, start_fen
        );
    }
}
