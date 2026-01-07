use std::time::{Duration, Instant};

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

pub struct Timer {
    start_time: Instant,
    max_duration: Duration,
    stopped: bool,
}

impl Timer {
    pub fn reset(duration: Duration) -> Self {
        Timer {
            start_time: Instant::now(),
            max_duration: duration,
            stopped: false,
        }
    }

    pub fn should_stop(&mut self, node_count: u64) -> bool {
        if node_count % 2048 == 0 {
            self.stopped = self.start_time.elapsed() >= self.max_duration;
        }
        self.stopped
    }
}

pub struct SearchContext {
    pub prev_pv: PvLine,
    pub node_count: u64,
    timer: Timer,
}

fn quiescence(
    position: &mut Position,
    mut alpha: i32,
    beta: i32,
    ply: u32,
    context: &mut SearchContext,
) -> i32 {
    if context.timer.should_stop(context.node_count) {
        return 0;
    }
    let stand_pat = evaluate(position);

    if stand_pat >= beta {
        return beta; // fail hard beta-cutoff
    }
    if stand_pat > alpha {
        alpha = stand_pat; // new lower bound -> pv move
    }

    let mut moves = position.generate_tactical_moves();

    // Move ordering
    order_moves_inplace(position, &mut moves, ply, context);
    for move_ in moves {
        let piece_at_target = position.board[move_.to];
        let original_castling_rights = position.castling_rights;
        let original_king_squares = position.king_squares;
        let original_ep_square = position.enpassant_square;
        let original_hash = position.hash;
        position.make_move(&move_);
        context.node_count += 1;
        let value = -quiescence(position, -beta, -alpha, ply + 1, context);
        position.unmake_move(
            &move_,
            piece_at_target,
            original_castling_rights,
            original_king_squares,
            original_ep_square,
        );
        position.hash = original_hash;
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
    context: &mut SearchContext,
) -> i32 {
    if context.timer.should_stop(context.node_count) {
        return 0;
    }

    if ply > 0 && position.is_repetition() {
        return 0;
    }

    // leaf node
    if depth == 0 {
        pv.clear();
        return quiescence(position, alpha, beta, ply + 1, context);
    }
    let mut line = PvLine::new(); // Local PV buffer for children
    let mut moves = position.generate_pseudo_moves();
    let mut found_legal_move = false;
    // Move ordering
    order_moves_inplace(position, &mut moves, ply, context);
    for move_ in moves {
        let piece_at_target = position.board[move_.to];
        let original_castling_rights = position.castling_rights;
        let original_king_squares = position.king_squares;
        let original_ep_square = position.enpassant_square;
        let original_hash = position.hash;
        position.make_move(&move_);

        // test legality
        position.is_white_turn = !position.is_white_turn; // consider from same side before move
        let idx = match position.is_white_turn {
            true => 0,
            false => 1,
        };
        let is_legal = if !is_square_attacked(position.king_squares[idx], position) {
            true
        } else { false };
        position.is_white_turn = !position.is_white_turn;

        if is_legal {
            found_legal_move = true;
            context.node_count += 1;
            // do not store to first rep index
            position.repetition_index += 1;
            position.repetition_stack[position.repetition_index] = position.hash;
            let value = -alphabeta(
                position,
                -beta,
                -alpha,
                depth - 1,
                ply + 1,
                &mut line,
                context,
            );
            position.unmake_move(
                &move_,
                piece_at_target,
                original_castling_rights,
                original_king_squares,
                original_ep_square,
            );
            position.hash = original_hash;
            position.repetition_index -= 1;

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
        } else {
            position.unmake_move(
                &move_,
                piece_at_target,
                original_castling_rights,
                original_king_squares,
                original_ep_square,
            );
            position.hash = original_hash;
        }
        
    }
    if !found_legal_move {
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

    alpha
}

pub fn search(position: &mut Position, depth: u32, timer: Timer) -> SearchContext {
    let mut context = SearchContext {
        prev_pv: PvLine::new(),
        node_count: 0,
        timer,
    };
    for d in 1..depth + 1 {
        let mut pv = PvLine::new();
        let alpha = -100000;
        let beta = 100000;
        let ply = 0;
        let value = alphabeta(position, alpha, beta, d, ply, &mut pv, &mut context);

        if !context.timer.stopped {
            context.prev_pv = pv;
            let pv_string = pv
                .moves
                .iter()
                .flatten() // filters out None and unwraps Some
                .map(get_move_string)
                .collect::<Vec<_>>()
                .join(" ");
            println!(
                "info score cp {value} depth {d} nodes {} pv {pv_string}",
                context.node_count
            );
        }
    }
    context
}
