import torch
import torch.nn as nn
from pathlib import Path
import numpy as np


class NNUE(nn.Module):
    def __init__(self, input_size, hidden_size):
        super().__init__()
        self.input_layer, self.hidden_layer = (
            nn.Linear(input_size, hidden_size),
            nn.Linear(hidden_size, 1),
        )

    def forward(self, x):
        x = torch.clamp(self.input_layer(x), 0, 1)
        return torch.sigmoid(self.hidden_layer(x))


def fen_to_features(fen):
    "Convert FEN to list of active feature indices (768 sparse binary)"
    piece_idx = dict(P=0, N=1, B=2, R=3, Q=4, K=5, p=6, n=7, b=8, r=9, q=10, k=11)
    board = fen.split()[0]
    sq, feats = 0, []
    for ch in board:
        if ch == "/":
            continue
        elif ch.isdigit():
            sq += int(ch)
        else:
            feats.append(piece_idx[ch] * 64 + sq)
            sq += 1
    return feats


def precompute_sparse(data_path, out_path="nnue_data.npz", max_pieces=32):
    "Precompute sparse features as padded numpy array"
    lines = Path(data_path).read_text().strip().split("\n")
    n = len(lines)
    indices = np.full((n, max_pieces), -1, dtype=np.int16)
    results = np.empty(n, dtype=np.float32)
    for i, line in enumerate(lines):
        if i % 100000 == 0:
            print(f"Parsing data {i}/{len(lines)}")

        fen, result = line.rsplit(" [", 1)
        results[i] = float(result.rstrip("]"))

        # fen, epd_result = line.split(' "', 1)
        # epd_result = epd_result.rstrip('";')
        # if epd_result == "1-0":
        #     result = 1.0
        # elif epd_result == "0-1":
        #     result = 0.0
        # elif epd_result == "1/2-1/2":
        #     result = 0.5
        # else:
        #     raise ValueError(f"Invalid result: {epd_result}")
        # results[i] = float(result)

        feats = fen_to_features(fen)
        indices[i, : len(feats)] = feats
    np.savez(out_path, indices=indices, results=results)
    print(f"Saved {n} positions ({indices.nbytes/1e6:.0f}MB)")


def load_data(path="nnue_data.npz"):
    data = np.load(path)
    indices = data["indices"].astype(np.int64)
    results = data["results"]
    print(f"loaded {len(results)} positions")
    return torch.from_numpy(indices), torch.from_numpy(results)


def make_dense_batch(indices, device):
    bs, mp = indices.shape
    x = torch.zeros(bs, 768, device=device)
    mask = indices >= 0
    rows = torch.arange(bs, device=device).unsqueeze(1).expand_as(indices)[mask]
    x[rows, indices[mask]] = 1.0
    return x


class Trainer:
    def __init__(self, opt, sched, loss_fn, bs, device):
        self.opt = opt
        self.sched = sched
        self.loss_fn = loss_fn
        self.bs = bs
        self.device = device

    def run_train_epoch(self, model, train_idx, indices, results):
        model.train()
        shuffled_train = torch.randperm(len(train_idx))
        total_loss, count = 0.0, 0
        for i in range(0, len(train_idx), self.bs):
            idx = train_idx[shuffled_train[i : i + self.bs]]
            x = make_dense_batch(indices[idx].to(self.device), self.device)
            y = results[idx].unsqueeze(1).to(self.device)
            loss = self.loss_fn(model(x), y)
            self.opt.zero_grad()
            loss.backward()
            self.opt.step()
            total_loss += loss.item() * len(idx)
            count += len(idx)
        self.sched.step()
        return total_loss / count

    def run_validation_epoch(self, model, val_idx, indices, results):
        model.eval()
        total_loss = 0.0
        with torch.no_grad():
            for i in range(0, len(val_idx), self.bs):
                idx = val_idx[i : i + self.bs]
                x = make_dense_batch(indices[idx].to(self.device), self.device)
                y = results[idx].unsqueeze(1).to(self.device)
                total_loss += self.loss_fn(model(x), y).item() * len(idx)
        return total_loss / len(val_idx)


def train_nnue(model, indices, results, n_epochs=100, lr=1e-3, bs=16384):
    device = torch.device("cuda" if torch.cuda.is_available() else "cpu")
    model.to(device)

    # train/val split
    n = len(results)
    n_val = n // 10
    perm = torch.randperm(n)
    val_idx, train_idx = perm[:n_val], perm[n_val:]

    # setup trainer
    opt = torch.optim.Adam(model.parameters(), lr=lr, weight_decay=1e-4)
    sched = torch.optim.lr_scheduler.CosineAnnealingLR(opt, T_max=n_epochs)
    loss_fn = nn.MSELoss()
    trainer = Trainer(opt, sched, loss_fn, bs, device)

    # train for n_epochs
    best_val, best_state = float("inf"), None
    for epoch in range(n_epochs):
        train_loss = trainer.run_train_epoch(model, train_idx, indices, results)
        val_loss = trainer.run_validation_epoch(model, val_idx, indices, results)

        marker = " *" if val_loss < best_val else ""
        if val_loss < best_val:
            best_val = val_loss
            best_state = {k: v.cpu().clone() for k, v in model.state_dict().items()}
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


def export_weights(model, path="nnue.bin"):
    "Export NNUE weights to flat binary file for Rust inference"
    with open(path, "wb") as f:
        for name in ["input_layer", "hidden_layer"]:
            layer = getattr(model, name)
            f.write(layer.weight.data.numpy().tobytes())
            f.write(layer.bias.data.numpy().tobytes())


if __name__ == "__main__":
    # precompute_sparse("lichess-big3-resolved.book", "nnue_data.npz")

    indices, results = load_data("nnue_data.npz")

    model = NNUE(input_size=768, hidden_size=256)
    train_nnue(model, indices, results, n_epochs=1)

    export_weights(model, "nnue.bin")
