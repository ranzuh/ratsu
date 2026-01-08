use crate::{movegen::get_move_string, position::Position};
use std::time::Instant;

#[derive(Default, Debug)]
pub struct PerftCounts {
    pub castlings: u64,
    pub captures: u64,
    pub enpassants: u64,
    pub promotions: u64,
}

pub fn perft(depth: u32, position: &mut Position, counts: &mut PerftCounts, divide: bool) -> u64 {
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
        position.make_move(&move_, depth);

        let result = perft(depth - 1, position, counts, false);
        if divide {
            println!("{} {}", get_move_string(&move_), result);
        }

        nodes += result;
        position.unmake_move(&move_, depth);
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
