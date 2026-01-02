use rustchess::{START_POSITION_FEN, position::Position, uci::handle_position};

#[test]
fn test_incremental_hash_changes() {
    // start from startpos and make moves incrementally
    let mut pos = Position::from_fen(START_POSITION_FEN);
    let inp = "position startpos moves d2d4 g8f6 b1d2 d7d5 b2b4 c8f5 b4b5 c7c5";
    handle_position(inp, &mut pos);

    // directly generate hash for the resulting position
    let pos_fen =
        Position::from_fen("rn1qkb1r/pp2pppp/5n2/1Ppp1b2/3P4/8/P1PNPPPP/R1BQKBNR w KQkq c6 0 5");

    // check that these match
    assert_eq!(pos.hash, pos_fen.hash)
}
