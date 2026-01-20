use std::{
    cmp::min,
    time::{Duration, Instant},
};

use crate::{
    evaluation::evaluate,
    hash::{NodeType, TranspositionTable},
    movegen::{Move, get_move_string, is_square_attacked},
    moveordering::{self},
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

pub struct Search<'a> {
    position: &'a mut Position,
    tt: &'a mut TranspositionTable,
    node_count: u64,
    timer: Timer,
    prev_pv: Vec<Move>,
    history: [[u32; 128]; 128],
    killers: [[Option<Move>; 2]; 64],
}

impl<'a> Search<'a> {
    pub fn run(
        position: &'a mut Position,
        tt: &'a mut TranspositionTable,
        depth: u32,
        movetime: u64,
    ) -> (Vec<Move>, u64) {
        tt.clear();
        let max_duration = Duration::from_millis(movetime);
        let mut search = Self {
            position,
            tt,
            node_count: 0,
            timer: Timer::new(max_duration),
            prev_pv: Vec::new(),
            history: [[0u32; 128]; 128],
            killers: [[None; 2]; 64],
        };
        search.search(depth)
    }

    fn search(&mut self, depth: u32) -> (Vec<Move>, u64) {
        for d in 1..depth + 1 {
            let mut pv: Vec<Move> = Vec::new();
            let alpha = -1000000;
            let beta = 1000000;
            let ply = 0;
            let follow_pv = true;
            let value = self.alphabeta(alpha, beta, d, ply, &mut pv, follow_pv);

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
                    self.node_count,
                    (self.node_count as f32 / self.timer.elapsed().as_secs_f32()) as u64
                );
            }
        }
        (self.prev_pv.clone(), self.node_count)
    }

    fn order_moves_inplace(&self, moves: &mut [Move], ply: u32, tt_move: Option<&Move>) {
        let pv_move = self.prev_pv.get(ply as usize);
        moveordering::order_moves_inplace(
            self.position,
            moves,
            ply,
            pv_move,
            tt_move,
            self.killers,
            self.history,
        );
    }

    fn quiescence(&mut self, mut alpha: i32, beta: i32, ply: u32) -> i32 {
        if self.timer.should_stop(self.node_count) {
            return 0;
        }
        let stand_pat = evaluate(self.position);

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
            self.position.make_move(&move_, ply);
            self.node_count += 1;
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
        if ply > 0 && self.timer.should_stop(self.node_count) {
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
        if depth >= 3 && !in_check && ply > 0 && !pv_node {
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

        let mut moves = self.position.generate_pseudo_moves();
        let mut node_type = NodeType::AlphaBound;
        let mut best_move: Option<Move> = None;
        let mut follow_pv = true;
        let mut legal_moves = 0;
        // Move ordering
        self.order_moves_inplace(&mut moves, ply, tt_move);
        for move_ in moves {
            self.position.make_move(&move_, ply);

            if is_legal(self.position) {
                legal_moves += 1;
                // Local PV buffer for children
                let mut line = Vec::new();
                self.node_count += 1;
                // do not store to first rep index
                self.position.repetition_index += 1;
                self.position.repetition_stack[self.position.repetition_index] = self.position.hash;
                let mut value;

                // Principal variation search
                if legal_moves == 1 {
                    // Search PV move with full window
                    value =
                        -self.alphabeta(-beta, -alpha, depth - 1, ply + 1, &mut line, follow_pv);
                } else {
                    // Search other moves with null window
                    value = -self.alphabeta(
                        -alpha - 1,
                        -alpha,
                        depth - 1,
                        ply + 1,
                        &mut line,
                        follow_pv,
                    );
                    if value > alpha && value < beta {
                        // didn't stay inside the window
                        // need to re-search with full window
                        value = -self.alphabeta(
                            -beta,
                            -alpha,
                            depth - 1,
                            ply + 1,
                            &mut line,
                            follow_pv,
                        );
                    }
                }

                self.position.unmake_move(&move_, ply);
                self.position.repetition_index -= 1;
                follow_pv = false;

                if value >= beta {
                    if !move_.is_capture {
                        let hist_score = self.history[move_.from][move_.to];
                        self.history[move_.from][move_.to] =
                            min(hist_score + depth * depth, 999999);
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
                }
            } else {
                self.position.unmake_move(&move_, ply);
            }
        }
        if legal_moves == 0 {
            if in_check {
                return -50000;
            } else {
                return 0;
            }
        }
        self.tt
            .write_entry(self.position.hash, alpha, node_type, depth, best_move);
        alpha
    }
}
