from dataclasses import dataclass

import torch
import torch.nn as nn
from pathlib import Path
import numpy as np


class NNUE(nn.Module):
    def __init__(self, input_size, acc_size):
        super().__init__()
        self.acc_layer = nn.Linear(input_size, acc_size)
        self.out_layer = nn.Linear(acc_size * 2, 1)

    def forward(self, w_x, b_x, is_white):
        w_acc = torch.clamp(self.acc_layer(w_x), 0, 1)
        b_acc = torch.clamp(self.acc_layer(b_x), 0, 1)

        # Concatenate STM first, then opponent
        stm_acc = torch.where(is_white.unsqueeze(1), w_acc, b_acc)
        opp_acc = torch.where(is_white.unsqueeze(1), b_acc, w_acc)
        combined = torch.cat([stm_acc, opp_acc], dim=1)  # [bs, 512]

        return torch.sigmoid(self.out_layer(combined))


def fen_to_features(fen):
    "Returns (white_features, black_features, is_white_turn)"
    piece_idx = dict(P=0, N=1, B=2, R=3, Q=4, K=5, p=6, n=7, b=8, r=9, q=10, k=11)
    # Black perspective swaps friendly/enemy and mirrors rank
    flip_idx = dict(p=0, n=1, b=2, r=3, q=4, k=5, P=6, N=7, B=8, R=9, Q=10, K=11)

    parts = fen.split()
    board = parts[0]
    is_white = parts[1] == "w"

    sq = 0
    w_feats, b_feats = [], []
    for ch in board:
        if ch == "/":
            continue
        elif ch.isdigit():
            sq += int(ch)
        else:
            # White perspective: normal
            w_feats.append(piece_idx[ch] * 64 + sq)
            # Black perspective: flip color mapping + mirror square
            row, col = divmod(sq, 8)
            mirrored_sq = (7 - row) * 8 + col
            b_feats.append(flip_idx[ch] * 64 + mirrored_sq)
            sq += 1

    return w_feats, b_feats, is_white


def precompute_sparse(data_path, out_path="nnue_data.npz", max_pieces=32):
    lines = Path(data_path).read_text().strip().split("\n")
    n = len(lines)
    w_indices = np.full((n, max_pieces), -1, dtype=np.int16)
    b_indices = np.full((n, max_pieces), -1, dtype=np.int16)
    stm = np.empty(n, dtype=np.bool_)
    results = np.empty(n, dtype=np.float32)

    for i, line in enumerate(lines):
        if i % 100000 == 0:
            print(f"Parsing data {i}/{len(lines)}")

        fen, result = line.rsplit(" [", 1)
        results[i] = float(result.rstrip("]"))
        wf, bf, is_white = fen_to_features(fen)
        w_indices[i, : len(wf)] = wf
        b_indices[i, : len(bf)] = bf
        stm[i] = is_white

    np.savez(
        out_path, w_indices=w_indices, b_indices=b_indices, stm=stm, results=results
    )


@dataclass
class PositionData:
    w_idx: torch.Tensor  # [N, max_pieces] int16
    b_idx: torch.Tensor  # [N, max_pieces] int16
    stm: torch.Tensor  # [N] bool
    results: torch.Tensor  # [N] float32

    @staticmethod
    def load(path, device):
        d = np.load(path)
        pd = PositionData(
            w_idx=torch.from_numpy(d["w_indices"]).to(device),
            b_idx=torch.from_numpy(d["b_indices"]).to(device),
            stm=torch.from_numpy(d["stm"]).to(device),
            results=torch.from_numpy(d["results"]).to(device),
        )
        print(f"loaded {len(pd.results)} positions to {device}")
        return pd

    def _dense(self, indices):
        bs, mp = indices.shape
        x = torch.zeros(bs, 768, device=indices.device)
        mask = indices >= 0
        rows = (
            torch.arange(bs, device=indices.device)
            .unsqueeze(1)
            .expand_as(indices)[mask]
        )
        x[rows, indices[mask].long()] = 1.0
        return x

    def get_batch(self, idx):
        w_x = self._dense(self.w_idx[idx])
        b_x = self._dense(self.b_idx[idx])
        s = self.stm[idx]
        y = self.results[idx].unsqueeze(1)
        y = torch.where(s.unsqueeze(1), y, 1.0 - y)
        return w_x, b_x, s, y


def train_nnue(model, data: PositionData, n_epochs=100, lr=1e-3, bs=16384, wd=1e-5):
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
    best_val, best_state = float("inf"), None
    for epoch in range(n_epochs):
        # Train
        model.train()
        shuffled = torch.randperm(len(train_idx), device=device)
        total_loss, count = 0.0, 0
        for i in range(0, len(train_idx), bs):
            w_x, b_x, s, y = data.get_batch(train_idx[shuffled[i : i + bs]])
            loss = loss_fn(model(w_x, b_x, s), y)
            opt.zero_grad()
            loss.backward()
            opt.step()
            total_loss += loss.item() * len(y)
            count += len(y)
        sched.step()

        # Validate
        model.eval()
        val_loss = 0.0
        with torch.no_grad():
            for i in range(0, len(val_idx), bs):
                w_x, b_x, s, y = data.get_batch(val_idx[i : i + bs])
                val_loss += loss_fn(model(w_x, b_x, s), y).item() * len(y)
        val_loss /= len(val_idx)

        marker = ""
        if val_loss < best_val:
            best_val = val_loss
            best_state = {k: v.cpu().clone() for k, v in model.state_dict().items()}
            marker = " *"
        print(
            f"Epoch {epoch+1}/{n_epochs}  train: {total_loss/count:.6f}  val: {val_loss:.6f}{marker}"
        )

    # save best weights
    save_path = "nnue_best.pt"
    torch.save(best_state, save_path)
    print(f"Best val loss: {best_val:.6f}, best weights saved to: {save_path}")

    # load best weights to model and return it
    model.cpu().load_state_dict(best_state)
    return model


def export_weights(model, path="nnue.bin"):
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
    model = NNUE(input_size=768, acc_size=256)
    train_nnue(model, data, n_epochs=50, wd=1e-5)
    export_weights(model, "nnue_foo.bin")
