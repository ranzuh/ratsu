from dataclasses import dataclass

import torch
import torch.nn as nn
from pathlib import Path
import numpy as np


class NNUE(nn.Module):
    def __init__(self, input_size, acc_size) -> None:
        super().__init__()
        self.acc_layer = nn.Linear(input_size, acc_size)
        self.out_layer = nn.Linear(acc_size * 2, 1)

    def forward(self, w_x, b_x, is_white) -> torch.Tensor:
        w_acc = torch.clamp(self.acc_layer(w_x), 0, 1)
        b_acc = torch.clamp(self.acc_layer(b_x), 0, 1)

        # Concatenate STM first, then opponent
        stm_acc = torch.where(is_white.unsqueeze(1), w_acc, b_acc)
        opp_acc = torch.where(is_white.unsqueeze(1), b_acc, w_acc)
        combined = torch.cat([stm_acc, opp_acc], dim=1)  # [bs, 512]

        return torch.sigmoid(self.out_layer(combined))


def fen_to_features(fen: str) -> tuple[list[int], list[int], bool]:
    piece_idx = dict(P=0, N=1, B=2, R=3, Q=4, K=5, p=6, n=7, b=8, r=9, q=10, k=11)
    # Black perspective swaps friendly/enemy
    flip_idx = dict(p=0, n=1, b=2, r=3, q=4, k=5, P=6, N=7, B=8, R=9, Q=10, K=11)

    parts = fen.split()
    board = parts[0]
    is_white = parts[1] == "w"

    square = 0
    w_feats = []
    b_feats = []
    for char in board:
        if char == "/":
            continue
        elif char.isdigit():
            square += int(char)
        else:
            # White perspective: normal
            w_feats.append(piece_idx[char] * 64 + square)
            # Black perspective: flip color mapping + mirror square
            row, col = divmod(square, 8)
            mirrored_square = (7 - row) * 8 + col
            b_feats.append(flip_idx[char] * 64 + mirrored_square)
            square += 1

    return w_feats, b_feats, is_white


def precompute_sparse(data_path: str, out_path: str) -> None:
    lines = Path(data_path).read_text().strip().split("\n")
    n = len(lines)
    max_pieces = 32  # chess board can hold max 32 pieces
    w_indices = np.full((n, max_pieces), -1, dtype=np.int16)
    b_indices = np.full((n, max_pieces), -1, dtype=np.int16)
    side_to_move = np.empty(n, dtype=np.bool_)
    results = np.empty(n, dtype=np.float32)

    for i, line in enumerate(lines):
        if i % 100000 == 0:
            print(f"Parsing data {i}/{len(lines)}")

        fen, result = line.rsplit(" [", 1)
        results[i] = float(result.rstrip("]"))
        w_feats, b_feats, is_white = fen_to_features(fen)
        w_indices[i, : len(w_feats)] = w_feats
        b_indices[i, : len(b_feats)] = b_feats
        side_to_move[i] = is_white

    np.savez(
        out_path,
        w_indices=w_indices,
        b_indices=b_indices,
        side_to_move=side_to_move,
        results=results,
    )


@dataclass
class PositionData:
    white_indices: torch.Tensor  # [N, max_pieces] int16
    black_indices: torch.Tensor  # [N, max_pieces] int16
    side_to_move: torch.Tensor  # [N] bool
    results: torch.Tensor  # [N] float32

    @staticmethod
    def load(path, device) -> "PositionData":
        data = np.load(path)
        pos_data = PositionData(
            white_indices=torch.from_numpy(data["w_indices"]).to(device),
            black_indices=torch.from_numpy(data["b_indices"]).to(device),
            side_to_move=torch.from_numpy(data["side_to_move"]).to(device),
            results=torch.from_numpy(data["results"]).to(device),
        )
        print(f"loaded {len(pos_data.results)} positions to {device}")
        return pos_data

    def _dense(self, sparse_idx: torch.Tensor) -> torch.Tensor:
        batch_size, _ = sparse_idx.shape
        x = torch.zeros(batch_size, 768, device=sparse_idx.device)
        mask = sparse_idx >= 0
        valid_rows = (
            torch.arange(batch_size, device=sparse_idx.device)
            .unsqueeze(1)
            .expand_as(sparse_idx)[mask]
        )
        valid_feats = sparse_idx[mask].long()
        x[valid_rows, valid_feats] = 1.0
        return x

    def get_batch(
        self, batch_indices: torch.Tensor
    ) -> tuple[torch.Tensor, torch.Tensor, torch.Tensor, torch.Tensor]:
        w_x = self._dense(self.white_indices[batch_indices])
        b_x = self._dense(self.black_indices[batch_indices])
        s = self.side_to_move[batch_indices]
        y = self.results[batch_indices].unsqueeze(1)
        y = torch.where(s.unsqueeze(1), y, 1.0 - y)
        return w_x, b_x, s, y


def train_nnue(
    model: NNUE,
    data: PositionData,
    n_epochs: int = 100,
    lr: float = 1e-3,
    bs: int = 16384,
    wd: float = 1e-5,
) -> NNUE:
    device = data.results.device
    model.to(device)

    # train/val split
    n = len(data.results)
    n_val = n // 10
    perm = torch.randperm(n, device=device)
    val_idx, train_idx = perm[:n_val], perm[n_val:]

    # setup trainer
    opt = torch.optim.Adam(model.parameters(), lr=lr, weight_decay=wd)
    sched = torch.optim.lr_scheduler.CosineAnnealingLR(opt, T_max=n_epochs)
    loss_fn = nn.MSELoss()

    # train for n_epochs
    best_val = float("inf")
    best_state = None
    for epoch in range(n_epochs):
        # Train
        model.train()
        shuffled = torch.randperm(len(train_idx), device=device)
        train_loss = 0.0
        for i in range(0, len(train_idx), bs):
            batch_indices = train_idx[shuffled[i : i + bs]]
            w_x, b_x, s, y = data.get_batch(batch_indices)
            loss = loss_fn(model(w_x, b_x, s), y)
            opt.zero_grad()
            loss.backward()
            opt.step()
            train_loss += loss.item() * len(y)
        sched.step()
        train_loss /= len(train_idx)

        # Validate
        model.eval()
        val_loss = 0.0
        with torch.no_grad():
            for i in range(0, len(val_idx), bs):
                batch_indices = val_idx[i : i + bs]
                w_x, b_x, s, y = data.get_batch(batch_indices)
                val_loss += loss_fn(model(w_x, b_x, s), y).item() * len(y)
        val_loss /= len(val_idx)

        marker = ""
        if val_loss < best_val:
            best_val = val_loss
            best_state = {k: v.cpu().clone() for k, v in model.state_dict().items()}
            marker = " *"
        print(
            f"Epoch {epoch+1}/{n_epochs}  train: {train_loss:.6f}  val: {val_loss:.6f}{marker}"
        )

    # save best weights
    save_path = "nnue_best.pt"
    torch.save(best_state, save_path)
    print(f"Best val loss: {best_val:.6f}, best weights saved to: {save_path}")

    # load best weights to model and return it
    model.cpu().load_state_dict(best_state)
    return model


def export_weights(model: NNUE, path: str) -> None:
    "Export NNUE weights to flat binary file for Rust inference"
    with open(path, "wb") as f:
        for name in ["acc_layer", "out_layer"]:
            layer = getattr(model, name)
            f.write(layer.weight.data.numpy().tobytes())
            f.write(layer.bias.data.numpy().tobytes())


if __name__ == "__main__":
    # precompute_sparse("lichess-big3-resolved.book", "nnue_data.npz")
    torch.manual_seed(42)
    device = torch.device("cuda" if torch.cuda.is_available() else "cpu")
    data = PositionData.load("nnue_data.npz", device=device)
    model = NNUE(input_size=768, acc_size=128)
    train_nnue(model, data, n_epochs=50, lr=0.00799, wd=2.37735e-06)
    export_weights(model, "nnue.bin")
