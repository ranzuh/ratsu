use crate::{
    evaluation::evaluate,
    movegen::{Move, get_move_string, is_square_attacked},
    moveordering::order_moves_inplace,
    position::Position,
};

const MAX_DEPTH: usize = 64;

struct PvLine {
    moves: [Option<Move>; MAX_DEPTH],
    count: usize,
}

impl PvLine {
    fn new() -> Self {
        PvLine {
            moves: [None; MAX_DEPTH],
            count: 0,
        }
    }

    fn clear(&mut self) {
        self.count = 0;
    }
}

fn quiescence(
    position: &mut Position,
    mut alpha: i32,
    beta: i32,
    nodecount: &mut u64,
) -> i32 {
    let stand_pat = evaluate(position);

    if stand_pat >= beta {
        return beta; // fail hard beta-cutoff
    }
    if stand_pat > alpha {
        alpha = stand_pat; // new lower bound -> pv move
    }

    let mut moves = position.generate_tactical_moves();

    // Move ordering
    order_moves_inplace(&position, &mut moves);
    for move_ in moves {
        let piece_at_target = position.board[move_.to];
        let original_castling_rights = position.castling_rights;
        let original_king_squares = position.king_squares;
        let original_ep_square = position.enpassant_square;
        position.make_move(&move_);
        *nodecount += 1;
        let value = -quiescence(position, -beta, -alpha, nodecount);
        position.unmake_move(
            &move_,
            piece_at_target,
            original_castling_rights,
            original_king_squares,
            original_ep_square,
        );
        if value >= beta {
            return beta; // fail hard beta-cutoff
        }
        if value > alpha {
            alpha = value; // new lower bound -> pv move
        }
    }
    alpha
}

fn alphabeta(
    position: &mut Position,
    mut alpha: i32,
    beta: i32,
    depth: u32,
    nodecount: &mut u64,
    pv: &mut PvLine,
) -> i32 {
    // leaf node
    if depth == 0 {
        pv.clear();
        return quiescence(position, alpha, beta, nodecount);
    }
    let mut line = PvLine::new(); // Local PV buffer for children
    let mut moves = position.generate_legal_moves();
    if moves.is_empty() {
        pv.clear();
        let idx = match position.is_white_turn {
            true => 0,
            false => 1,
        };
        if is_square_attacked(position.king_squares[idx], position) {
            return -50000;
        } else {
            return 0;
        }
    }
    // Move ordering
    order_moves_inplace(&position, &mut moves);
    for move_ in moves {
        let piece_at_target = position.board[move_.to];
        let original_castling_rights = position.castling_rights;
        let original_king_squares = position.king_squares;
        let original_ep_square = position.enpassant_square;
        position.make_move(&move_);
        *nodecount += 1;
        let value = -alphabeta(position, -beta, -alpha, depth - 1, nodecount, &mut line);
        position.unmake_move(
            &move_,
            piece_at_target,
            original_castling_rights,
            original_king_squares,
            original_ep_square,
        );
        if value >= beta {
            return beta; // fail hard beta-cutoff
        }
        if value > alpha {
            alpha = value; // new lower bound -> pv move

            // Update PV: prepend current move to child's PV
            pv.moves[0] = Some(move_);
            pv.moves[1..=line.count].copy_from_slice(&line.moves[..line.count]);
            pv.count = line.count + 1;
        }
    }
    alpha
}

pub fn search(position: &mut Position, depth: u32, nodecount: &mut u64) -> Move {
    let mut pv = PvLine::new();
    for d in 1..depth + 1 {
        let value = alphabeta(position, -100000, 100000, d, nodecount, &mut pv);
        let pv_string = pv.moves
            .iter()
            .flatten()  // filters out None and unwraps Some
            .map(|m| get_move_string(m))
            .collect::<Vec<_>>()
            .join(" ");
        println!(
            "info score cp {value} depth {d} nodes {nodecount} pv {pv_string}",
        );
    }
    pv.moves[0].expect("pv should have moves")
}
