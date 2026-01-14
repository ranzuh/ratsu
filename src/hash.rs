use crate::movegen::Move;

struct Xorshift64 {
    state: u64,
}

impl Xorshift64 {
    const fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next(&mut self) -> u64 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        self.state
    }
}

pub struct ZobristKeys {
    pub piece_keys: [[u64; 24]; 64],
    pub black_to_move_key: u64,
    pub castling_rights_keys: [u64; 4],
    pub enpassant_file_keys: [u64; 8],
}

impl ZobristKeys {
    pub fn new() -> Self {
        const SEED: u64 = 0x1EF105C43DEF1F9F;
        let mut state = Xorshift64::new(SEED);

        Self {
            piece_keys: std::array::from_fn(|_| std::array::from_fn(|_| state.next())),
            black_to_move_key: state.next(),
            castling_rights_keys: std::array::from_fn(|_| state.next()),
            enpassant_file_keys: std::array::from_fn(|_| state.next()),
        }
    }
}
#[derive(Debug, Default, Clone, PartialEq)]
pub enum NodeType {
    #[default]
    Exact,
    AlphaBound,
    BetaBound,
}

#[derive(Debug, Default, Clone)]
pub struct TTEntry {
    hash_key: u64,
    score: i32,
    node_type: NodeType,
    depth: u32,
    best_move: Option<Move>,
}

pub struct TranspositionTable {
    pub entries: Vec<TTEntry>,
    pub size: usize,
}

impl TranspositionTable {
    pub fn new(size_mb: usize) -> Self {
        let size = (size_mb * 1024 * 1024) / std::mem::size_of::<TTEntry>();
        Self {
            entries: vec![TTEntry::default(); size],
            size,
        }
    }
    pub fn clear(&mut self) {
        for e in &mut self.entries {
            e.hash_key = 0;
            e.score = 0;
            e.node_type = NodeType::Exact;
            e.depth = 0;
        }
    }
    pub fn write_entry(
        &mut self,
        hash_key: u64,
        score: i32,
        node_type: NodeType,
        depth: u32,
        best_move: Option<Move>,
    ) {
        let idx = (hash_key as usize) % self.size;
        self.entries[idx].hash_key = hash_key;
        self.entries[idx].score = score;
        self.entries[idx].node_type = node_type;
        self.entries[idx].depth = depth;
        self.entries[idx].best_move = best_move;
    }
    pub fn read_entry(
        &self,
        hash_key: u64,
        alpha: i32,
        beta: i32,
        depth: u32,
    ) -> (Option<i32>, Option<Move>) {
        let idx = (hash_key as usize) % self.size;
        let entry = &self.entries[idx];
        if entry.hash_key == hash_key && entry.depth >= depth {
            if entry.node_type == NodeType::Exact {
                return (Some(entry.score), entry.best_move);
            }
            if entry.node_type == NodeType::BetaBound && entry.score >= beta {
                return (Some(beta), entry.best_move);
            }
            if entry.node_type == NodeType::AlphaBound && entry.score <= alpha {
                return (Some(alpha), entry.best_move);
            }
        }
        (None, entry.best_move)
    }
}
