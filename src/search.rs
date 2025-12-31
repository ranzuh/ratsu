use crate::{
    evaluation::evaluate,
    movegen::{Move, get_move_string, is_square_attacked},
    moveordering::order_moves_inplace,
    position::Position,
};

const MAX_DEPTH: usize = 64;

#[derive(Debug, Clone, Copy)]
pub struct PvLine {
    pub moves: [Option<Move>; MAX_DEPTH],
    pub count: usize,
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

pub struct SearchInfo {
    pub prev_pv: PvLine,
    pub node_count: u64,
}

fn quiescence(
    position: &mut Position,
    mut alpha: i32,
    beta: i32,
    ply: u32,
    info: &mut SearchInfo,
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
    order_moves_inplace(&position, &mut moves, ply, info);
    for move_ in moves {
        let piece_at_target = position.board[move_.to];
        let original_castling_rights = position.castling_rights;
        let original_king_squares = position.king_squares;
        let original_ep_square = position.enpassant_square;
        position.make_move(&move_);
        info.node_count += 1;
        let value = -quiescence(position, -beta, -alpha, ply + 1, info);
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
    ply: u32,
    pv: &mut PvLine,
    info: &mut SearchInfo,
) -> i32 {
    // leaf node
    if depth == 0 {
        pv.clear();
        return quiescence(position, alpha, beta, ply + 1, info);
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
    order_moves_inplace(&position, &mut moves, ply, info);
    for move_ in moves {
        let piece_at_target = position.board[move_.to];
        let original_castling_rights = position.castling_rights;
        let original_king_squares = position.king_squares;
        let original_ep_square = position.enpassant_square;
        position.make_move(&move_);
        info.node_count += 1;
        let value = -alphabeta(position, -beta, -alpha, depth - 1, ply + 1, &mut line, info);
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

pub fn search(position: &mut Position, depth: u32) -> SearchInfo {
    let mut info = SearchInfo {
        prev_pv: PvLine::new(),
        node_count: 0,
    };
    for d in 1..depth + 1 {
        let mut pv = PvLine::new();
        let alpha = -100000;
        let beta = 100000;
        let ply = 0;
        let value = alphabeta(position, alpha, beta, d, ply, &mut pv, &mut info);

        info.prev_pv = pv.clone();

        let pv_string = pv
            .moves
            .iter()
            .flatten() // filters out None and unwraps Some
            .map(|m| get_move_string(m))
            .collect::<Vec<_>>()
            .join(" ");
        println!(
            "info score cp {value} depth {d} nodes {} pv {pv_string}",
            info.node_count
        );
    }
    info
}
