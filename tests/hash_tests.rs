use rustchess::{START_POSITION_FEN, position::Position, uci::handle_position};

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
