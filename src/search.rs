use std::{
    cmp::max,
    cmp::min,
    time::{Duration, Instant},
};

use crate::{
    // evaluation::evaluate,
    hash::{NodeType, TranspositionTable},
    movegen::{Move, get_move_string, is_square_attacked},
    moveordering::{self},
    nnue::{Nnue, nnue_evaluate},
    position::Position,
};

pub struct Timer {
    start_time: Instant,
    max_duration: Duration,
    stopped: bool,
}

impl Timer {
    pub fn new(max_duration: Duration) -> Self {
        Self {
            start_time: Instant::now(),
            max_duration,
            stopped: false,
        }
    }

    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    pub fn should_stop(&mut self, node_count: u64) -> bool {
        if node_count % 2048 == 0 {
            self.stopped = self.start_time.elapsed() >= self.max_duration;
        }
        self.stopped
    }
}

pub fn is_legal(position: &mut Position) -> bool {
    position.is_white_turn = !position.is_white_turn; // consider from same side before move
    let idx = if position.is_white_turn { 0 } else { 1 };
    let is_legal = !is_square_attacked(position.king_squares[idx], position);
    position.is_white_turn = !position.is_white_turn;
    is_legal
}

#[allow(dead_code)]
fn print_search_stats(stats: &SearchStats) {
    let first_move_cut_pct = if stats.cut_nodes > 0 {
        stats.first_move_cuts as f64 / stats.cut_nodes as f64 * 100.0
    } else {
        0.0
    };
    let avg_cut_move_index = if stats.cut_nodes > 0 {
        stats.cut_move_index_sum as f64 / stats.cut_nodes as f64
    } else {
        0.0
    };
    println!(
        "info first_move_cut_pct {:.2} avg_cut_move_index {:.2} pvnodes {} cutnodes {} allnodes {}",
        first_move_cut_pct, avg_cut_move_index, stats.pv_nodes, stats.cut_nodes, stats.all_nodes
    );
}

#[derive(Default, Debug)]
struct SearchStats {
    node_count: u64,
    pv_nodes: u64,
    cut_nodes: u64,
    all_nodes: u64,
    first_move_cuts: u64,
    cut_move_index_sum: u64,
}

pub struct Search<'a> {
    position: &'a mut Position,
    tt: &'a mut TranspositionTable,
    stats: SearchStats,
    timer: Timer,
    prev_pv: Vec<Move>,
    history: [[u32; 128]; 128],
    killers: [[Option<Move>; 2]; 64],
    use_pruning: bool,
    nnue: &'a Nnue,
}

impl<'a> Search<'a> {
    pub fn run(
        position: &'a mut Position,
        tt: &'a mut TranspositionTable,
        depth: u32,
        movetime: u64,
        use_pruning: bool,
        nnue: &'a Nnue,
    ) -> (Vec<Move>, u64) {
        tt.clear();
        let max_duration = Duration::from_millis(movetime);
        let mut search = Self {
            position,
            tt,
            stats: SearchStats::default(),
            timer: Timer::new(max_duration),
            prev_pv: Vec::new(),
            history: [[0u32; 128]; 128],
            killers: [[None; 2]; 64],
            use_pruning: use_pruning,
            nnue,
        };
        search.search(depth)
    }

    fn search(&mut self, depth: u32) -> (Vec<Move>, u64) {
        let mut prev_score = 0;
        for d in 1..depth + 1 {
            let mut pv: Vec<Move> = Vec::new();

            // Aspiration windows
            let mut delta = 25;
            let mut alpha = prev_score - delta;
            let mut beta = prev_score + delta;

            let mut value = 0;

            // Widen the aspiration window until the score is within the bounds or we hit a time limit
            while !self.timer.stopped {
                let ply = 0;
                value = self.alphabeta(alpha, beta, d, ply, &mut pv, true);

                if value <= alpha {
                    alpha = max(alpha - delta, -1000000);
                    delta *= 2;
                } else if value >= beta {
                    beta = min(beta + delta, 1000000);
                    delta *= 2;
                } else {
                    prev_score = value;
                    break;
                }
            }

            if !self.timer.stopped {
                self.prev_pv = pv.clone();
                let pv_string = pv
                    .clone()
                    .iter()
                    .map(get_move_string)
                    .collect::<Vec<_>>()
                    .join(" ");
                println!(
                    "info depth {d} score cp {value} nodes {} nps {} pv {pv_string}",
                    self.stats.node_count,
                    (self.stats.node_count as f32 / self.timer.elapsed().as_secs_f32()) as u64
                );
                // print_search_stats(&self.stats);
            }
        }
        (self.prev_pv.clone(), self.stats.node_count)
    }

    fn order_moves_inplace(&self, moves: &mut [Move], ply: u32, tt_move: Option<&Move>) {
        moveordering::order_moves_inplace(
            self.position,
            moves,
            ply,
            tt_move,
            &self.killers,
            &self.history,
        );
    }

    fn quiescence(&mut self, mut alpha: i32, beta: i32, ply: u32) -> i32 {
        self.stats.node_count += 1;

        if self.timer.should_stop(self.stats.node_count) {
            return 0;
        }
        let stand_pat = nnue_evaluate(self.position, self.nnue, ply);

        if stand_pat >= beta {
            return beta; // fail hard beta-cutoff
        }
        if stand_pat > alpha {
            alpha = stand_pat; // new lower bound -> pv move
        }

        let mut moves = self.position.generate_tactical_moves();

        // Move ordering
        self.order_moves_inplace(&mut moves, ply, None);
        for move_ in moves {
            self.position.make_move(&move_, ply, self.nnue);

            if !is_legal(self.position) {
                self.position.unmake_move(&move_, ply);
                continue;
            }

            let value = -self.quiescence(-beta, -alpha, ply + 1);
            self.position.unmake_move(&move_, ply);
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
        &mut self,
        mut alpha: i32,
        beta: i32,
        mut depth: u32,
        ply: u32,
        pv: &mut Vec<Move>,
        pv_node: bool,
    ) -> i32 {
        self.stats.node_count += 1;

        if ply > 0 && self.timer.should_stop(self.stats.node_count) {
            return 0;
        }

        if ply > 0 && self.position.is_repetition() {
            return 0;
        }
        if self.position.fifty >= 100 {
            return 0;
        }

        // check extension
        let idx = match self.position.is_white_turn {
            true => 0,
            false => 1,
        };
        let in_check = is_square_attacked(self.position.king_squares[idx], self.position);

        if in_check {
            depth += 1;
        }

        // null move pruning
        if self.use_pruning && depth >= 3 && !in_check && ply > 0 && !pv_node {
            let copy_ep = self.position.enpassant_square;
            self.position.make_null();

            let mut line = Vec::new();
            let value = -self.alphabeta(-beta, -beta + 1, depth - 3, ply + 1, &mut line, false);
            self.position.unmake_null(copy_ep);

            if value >= beta {
                return beta;
            }
        }

        // leaf node
        if depth == 0 {
            // TODO: Maybe not pass history and killers to quiesc? maybe just sort using mvv lva in there?
            return self.quiescence(alpha, beta, ply + 1);
        }

        let mut tt_move: Option<&Move> = None;
        if ply > 0 {
            let (tt_value, _tt_move) = self.tt.read_entry(self.position.hash, alpha, beta, depth);
            if let Some(value) = tt_value {
                return value;
            }
            tt_move = _tt_move;
        }

        // Futility pruning condition
        let mut futility_prune = false;
        if self.use_pruning && depth <= 2 && !in_check && !pv_node {
            let eval = nnue_evaluate(self.position, self.nnue, ply);
            let margin = if depth == 1 { 100 } else { 300 };
            if eval + margin < alpha {
                futility_prune = true;
            }
        }

        let mut moves = self.position.generate_pseudo_moves();
        let mut node_type = NodeType::AlphaBound;
        let mut best_move: Option<Move> = None;
        let mut legal_moves = 0;
        // Move ordering
        self.order_moves_inplace(&mut moves, ply, tt_move);
        for move_ in moves {
            self.position.make_move(&move_, ply, self.nnue);

            if !is_legal(self.position) {
                self.position.unmake_move(&move_, ply);
                continue;
            }

            legal_moves += 1;
            // Local PV buffer for children
            let mut line = Vec::new();

            // do not store to first rep index
            self.position.repetition_index += 1;
            self.position.repetition_stack[self.position.repetition_index] = self.position.hash;
            let mut value;

            // Principal variation search
            if legal_moves == 1 {
                // Search PV move with full window
                value = -self.alphabeta(-beta, -alpha, depth - 1, ply + 1, &mut line, pv_node);
            } else {
                // Search other moves with null window

                if futility_prune && !move_.is_capture && !move_.promoted_piece.is_some() {
                    self.position.unmake_move(&move_, ply);
                    self.position.repetition_index -= 1;
                    continue;
                }

                let mut reduction = 0;
                if depth >= 3
                    && legal_moves > 4
                    && !in_check
                    && !pv_node
                    && !move_.is_capture
                    && !move_.promoted_piece.is_some()
                {
                    reduction = 1;
                    // if legal_moves > 6 {
                    //     reduction = depth / 3;
                    // }
                }
                value = -self.alphabeta(
                    -alpha - 1,
                    -alpha,
                    depth - 1 - reduction,
                    ply + 1,
                    &mut line,
                    false,
                );

                if reduction > 0 && value > alpha {
                    value =
                        -self.alphabeta(-alpha - 1, -alpha, depth - 1, ply + 1, &mut line, false);
                }

                if value > alpha && value < beta {
                    // didn't stay inside the window
                    // need to re-search with full window
                    value = -self.alphabeta(-beta, -alpha, depth - 1, ply + 1, &mut line, true);
                }
            }

            self.position.unmake_move(&move_, ply);
            self.position.repetition_index -= 1;

            if value >= beta {
                if !move_.is_capture {
                    let hist_score = self.history[move_.from][move_.to];
                    self.history[move_.from][move_.to] = min(hist_score + depth * depth, 999999);
                    self.killers[ply as usize][1] = self.killers[ply as usize][0];
                    self.killers[ply as usize][0] = Some(move_);
                }
                self.tt.write_entry(
                    self.position.hash,
                    beta,
                    NodeType::BetaBound,
                    depth,
                    Some(move_),
                );

                self.stats.cut_nodes += 1;
                self.stats.cut_move_index_sum += (legal_moves - 1) as u64;
                if legal_moves == 1 {
                    self.stats.first_move_cuts += 1;
                }

                return beta; // fail hard beta-cutoff
            }
            if value > alpha {
                alpha = value; // new lower bound -> pv move

                // Update PV: prepend current move to child's PV
                pv.clear();
                pv.push(move_);
                pv.append(&mut line);

                node_type = NodeType::Exact;
                best_move = Some(move_);

                self.stats.pv_nodes += 1;
            }
        }
        if legal_moves == 0 {
            if in_check {
                return -50000 + ply as i32;
            } else {
                return 0;
            }
        }
        self.tt
            .write_entry(self.position.hash, alpha, node_type, depth, best_move);

        self.stats.all_nodes += 1;

        alpha
    }
}
