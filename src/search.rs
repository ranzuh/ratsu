use std::time::{Duration, Instant};

use crate::{
    evaluation::evaluate,
    hash::{NodeType, TranspositionTable},
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
    history: &mut [[u32; 128]; 128],
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
    order_moves_inplace(position, &mut moves, ply, context, history, None);
    for move_ in moves {
        position.make_move(&move_, ply);
        context.node_count += 1;
        let value = -quiescence(position, -beta, -alpha, ply + 1, context, history);
        position.unmake_move(&move_, ply);
        if value >= beta {
            return beta; // fail hard beta-cutoff
        }
        if value > alpha {
            alpha = value; // new lower bound -> pv move
        }
    }
    alpha
}

fn is_legal(position: &mut Position) -> bool {
    position.is_white_turn = !position.is_white_turn; // consider from same side before move
    let idx = if position.is_white_turn { 0 } else { 1 };
    let is_legal = !is_square_attacked(position.king_squares[idx], position);
    position.is_white_turn = !position.is_white_turn;
    is_legal
}

fn alphabeta(
    position: &mut Position,
    mut alpha: i32,
    beta: i32,
    mut depth: u32,
    ply: u32,
    pv: &mut PvLine,
    context: &mut SearchContext,
    history: &mut [[u32; 128]; 128],
    tt: &mut TranspositionTable,
) -> i32 {
    if ply > 0 && context.timer.should_stop(context.node_count) {
        return 0;
    }

    if ply > 0 && position.is_repetition() {
        return 0;
    }

    // check extension
    let idx = match position.is_white_turn {
        true => 0,
        false => 1,
    };
    let in_check = is_square_attacked(position.king_squares[idx], position);

    if in_check {
        depth += 1;
    }

    // leaf node
    if depth == 0 {
        pv.clear();
        return quiescence(position, alpha, beta, ply + 1, context, history);
    }

    let mut tt_move: Option<Move> = None;
    if ply > 0 {
        let (tt_value, _tt_move) = tt.read_entry(position.hash, alpha, beta, depth);
        if let Some(value) = tt_value {
            return value;
        }
        tt_move = _tt_move;
    }

    let mut line = PvLine::new(); // Local PV buffer for children
    let mut moves = position.generate_pseudo_moves();
    let mut found_legal_move = false;
    let mut node_type = NodeType::AlphaBound;
    let mut best_move: Option<Move> = None;
    // Move ordering
    order_moves_inplace(position, &mut moves, ply, context, history, tt_move);
    for move_ in moves {
        position.make_move(&move_, ply);

        if is_legal(position) {
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
                history,
                tt,
            );
            position.unmake_move(&move_, ply);
            position.repetition_index -= 1;

            if value >= beta {
                if !move_.is_capture {
                    history[move_.from][move_.to] += depth * depth;
                }
                tt.write_entry(position.hash, beta, NodeType::BetaBound, depth, Some(move_));
                return beta; // fail hard beta-cutoff
            }
            if value > alpha {
                alpha = value; // new lower bound -> pv move

                // Update PV: prepend current move to child's PV
                pv.moves[0] = Some(move_);
                pv.moves[1..=line.count].copy_from_slice(&line.moves[..line.count]);
                pv.count = line.count + 1;

                node_type = NodeType::Exact;
                best_move = Some(move_);
            }
        } else {
            position.unmake_move(&move_, ply);
        }
    }
    if !found_legal_move {
        pv.clear();
        if in_check {
            return -50000;
        } else {
            return 0;
        }
    }
    tt.write_entry(position.hash, alpha, node_type, depth, best_move);
    alpha
}

pub fn search(
    position: &mut Position,
    depth: u32,
    timer: Timer,
    tt: &mut TranspositionTable,
) -> SearchContext {
    let mut context = SearchContext {
        prev_pv: PvLine::new(),
        node_count: 0,
        timer,
    };
    tt.clear();

    for d in 1..depth + 1 {
        let mut pv = PvLine::new();
        let alpha = -100000;
        let beta = 100000;
        let ply = 0;
        let mut history = [[0u32; 128]; 128];
        let value = alphabeta(
            position,
            alpha,
            beta,
            d,
            ply,
            &mut pv,
            &mut context,
            &mut history,
            tt,
        );

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
