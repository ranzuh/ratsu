use crate::{Position, movegen::get_move_string};
use std::time::Instant;

#[derive(Default, Debug)]
pub struct PerftCounts {
    pub castlings: u64,
    pub captures: u64,
    pub enpassants: u64,
    pub promotions: u64,
}

fn perft(depth: u32, position: &mut Position, counts: &mut PerftCounts, divide: bool) -> u64 {
    let mut nodes: u64 = 0;

    if depth == 0 {
        return 1;
    }

    let moves = position.generate_legal_moves();
    for move_ in moves {
        // TODO: remove later? - debug stuff
        if move_.is_castling {
            counts.castlings += 1;
        }
        if move_.is_capture {
            counts.captures += 1;
        }
        if move_.is_enpassant {
            counts.enpassants += 1;
        }
        if move_.promoted_piece.is_some() {
            counts.promotions += 1;
        }
        //let mut pos_copy = position.clone();
        let target_piece = position.board[move_.to];
        let original_castling_rights = position.castling_rights;
        let original_king_squares = position.king_squares;
        let original_ep_square = position.enpassant_square;
        position.make_move(&move_);

        let result = perft(depth - 1, position, counts, false);
        if divide {
            println!("{} {}", get_move_string(&move_), result);
        }

        nodes += result;
        //*position = pos_copy;
        position.unmake_move(
            &move_,
            target_piece,
            original_castling_rights,
            original_king_squares,
            original_ep_square,
        );
    }
    nodes
}

pub fn run_perft(depth: u32, position: &mut Position) {
    let mut counts = PerftCounts::default();

    let start = Instant::now();
    let total_nodes = perft(depth, position, &mut counts, true);
    let duration = start.elapsed().as_secs_f32();
    let nodes_per_sec = (total_nodes as f32 / duration) as u64;

    println!("Perft depth {}: {}", depth, total_nodes);
    println!("Time taken: {}", duration);
    println!("NPS: {}", nodes_per_sec);
    println!("captures: {}", counts.captures);
    println!("castles: {}", counts.castlings);
    println!("enpassants: {}", counts.enpassants);
    println!("promotions: {}", counts.promotions);
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
