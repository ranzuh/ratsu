import time

import torch
import torch.nn as nn
from torch.utils.data import Dataset, DataLoader
from pathlib import Path
import numpy as np

PIECE_IDX = dict(P=0, N=1, B=2, R=3, Q=4, K=5, p=6, n=7, b=8, r=9, q=10, k=11)


def fen_to_features(fen):
    "Convert FEN to list of active feature indices (768 sparse binary)"
    board = fen.split()[0]
    sq, feats = 0, []
    for ch in board:
        if ch == "/":
            continue
        elif ch.isdigit():
            sq += int(ch)
        else:
            feats.append(PIECE_IDX[ch] * 64 + sq)
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


class NNUE(nn.Module):
    def __init__(self, n_in=768, n_h1=512, n_h2=32):
        super().__init__()
        self.l1, self.l2, self.l3 = (
            nn.Linear(n_in, n_h1),
            nn.Linear(n_h1, n_h2),
            nn.Linear(n_h2, 1),
        )

    def forward(self, x):
        x = torch.clamp(self.l1(x), 0, 1)
        x = torch.clamp(self.l2(x), 0, 1)
        return torch.sigmoid(self.l3(x))


class NNUEData(Dataset):
    "Load FEN + result training data for NNUE"

    def __init__(self, path):
        lines = Path(path).read_text().strip().split("\n")
        self.fens, self.results = [], []
        for line in lines:

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
            # self.results.append(float(result))

            fen, result = line.rsplit(" [", 1)
            self.results.append(float(result.rstrip("]")))

            self.fens.append(fen)

    def __len__(self):
        return len(self.fens)

    def __getitem__(self, i):
        feats = fen_to_features(self.fens[i])
        x = torch.zeros(768)
        x[feats] = 1.0
        return x, torch.tensor([self.results[i]], dtype=torch.float32)


class NNUEDataNP(Dataset):
    def __init__(self, path="nnue_data.npz"):
        data = np.load(path)
        self.indices, self.results = data["indices"], data["results"]

    def __len__(self):
        return len(self.results)

    def __getitem__(self, i):
        x = torch.zeros(768)
        idx = self.indices[i]
        x[idx[idx >= 0]] = 1.0
        return x, torch.tensor([self.results[i]])


# class NNUEDataFast(Dataset):
#     "Precomputed sparse NNUE training data"
#     def __init__(self, path):
#         lines = Path(path).read_text().strip().split('\n')
#         self.features, self.results = [], []
#         for i, line in enumerate(lines):
#             if i % 100_000 == 0: print(f"processing line {i}/{len(lines)}")
#             fen, result = line.rsplit(' [', 1)
#             self.results.append(float(result.rstrip(']')))
#             self.features.append(fen_to_features(fen))
#         self.results = torch.tensor(self.results, dtype=torch.float32).unsqueeze(1)

#     def __len__(self): return len(self.features)

#     def __getitem__(self, i):
#         x = torch.zeros(768)
#         x[self.features[i]] = 1.0
#         return x, self.results[i]

# def train_nnue(data_path, epochs=30, lr=1e-3, bs=4096):
#     device = torch.device('cuda' if torch.cuda.is_available() else 'cpu')
#     print(f"Using {device}")
#     ds = NNUEDataFast(data_path)
#     dl = DataLoader(ds, batch_size=bs, shuffle=True, num_workers=4, pin_memory=True)
#     model = NNUE().to(device)
#     opt = torch.optim.Adam(model.parameters(), lr=lr)
#     loss_fn = nn.MSELoss()
#     for epoch in range(epochs):
#         total_loss, n = 0.0, 0
#         for x, y in dl:
#             x, y = x.to(device, non_blocking=True), y.to(device, non_blocking=True)
#             loss = loss_fn(model(x), y)
#             opt.zero_grad()
#             loss.backward()
#             opt.step()
#             total_loss += loss.item() * len(x)
#             n += len(x)
#         print(f"Epoch {epoch+1}/{epochs}  loss: {total_loss/n:.6f}")
#     return model.cpu()


def load_data(path="nnue_data.npz"):
    data = np.load(path)
    return torch.from_numpy(data["indices"].astype(np.int64)), torch.from_numpy(
        data["results"]
    )


def make_dense_batch(indices, device):
    bs, mp = indices.shape
    x = torch.zeros(bs, 768, device=device)
    mask = indices >= 0
    rows = torch.arange(bs, device=device).unsqueeze(1).expand_as(indices)[mask]
    x[rows, indices[mask]] = 1.0
    return x


# def train_nnue(indices, results, epochs=100, lr=1e-3, bs=16384, load_model_path=None):
#     device = torch.device("cuda" if torch.cuda.is_available() else "cpu")
#     print(f"Using {device}")
#     n = len(results)
#     model = NNUE().to(device)
#     if load_model_path:
#         print("loading model:", load_model_path)
#         model.load_state_dict(torch.load(load_model_path))
#     opt = torch.optim.Adam(model.parameters(), lr=lr)
#     sched = torch.optim.lr_scheduler.CosineAnnealingLR(opt, T_max=epochs)
#     loss_fn = nn.MSELoss()
#     for epoch in range(epochs):
#         start = time.time()
#         perm = torch.randperm(n)
#         total_loss, n_seen = 0.0, 0
#         for i in range(0, n, bs):
#             idx = perm[i:i+bs]
#             x = make_dense_batch(indices[idx].to(device), device)
#             y = results[idx].unsqueeze(1).to(device)
#             loss = loss_fn(model(x), y)
#             opt.zero_grad()
#             loss.backward()
#             opt.step()
#             total_loss += loss.item() * len(idx)
#             n_seen += len(idx)
#         end = time.time()
#         avg_loss = total_loss / n_seen
#         sched.step()
#         print(f"Epoch {epoch+1}/{epochs}  loss: {avg_loss:.6f}  duration: {end-start:.2f}s")
#         # torch.save(model.state_dict(), f"epoch_{epoch+1}_loss_{avg_loss:.6f}nnue.pt")
#     return model.cpu()


def train_nnue(indices, results, epochs=100, lr=1e-3, bs=16384):
    device = torch.device("cuda" if torch.cuda.is_available() else "cpu")
    n = len(results)
    n_val = n // 10
    perm = torch.randperm(n)
    val_idx, train_idx = perm[:n_val], perm[n_val:]
    model = NNUE().to(device)
    opt = torch.optim.Adam(model.parameters(), lr=lr, weight_decay=1e-4)
    sched = torch.optim.lr_scheduler.CosineAnnealingLR(opt, T_max=epochs)
    loss_fn = nn.MSELoss()
    best_val, best_state = float("inf"), None
    for epoch in range(epochs):
        model.train()
        tp = torch.randperm(len(train_idx))
        total_loss, count = 0.0, 0
        for i in range(0, len(train_idx), bs):
            idx = train_idx[tp[i : i + bs]]
            x = make_dense_batch(indices[idx].to(device), device)
            y = results[idx].unsqueeze(1).to(device)
            loss = loss_fn(model(x), y)
            opt.zero_grad()
            loss.backward()
            opt.step()
            total_loss += loss.item() * len(idx)
            count += len(idx)
        sched.step()
        model.eval()
        with torch.no_grad():
            vl = 0.0
            for i in range(0, n_val, bs):
                idx = val_idx[i : i + bs]
                x = make_dense_batch(indices[idx].to(device), device)
                y = results[idx].unsqueeze(1).to(device)
                vl += loss_fn(model(x), y).item() * len(idx)
        vl /= n_val
        marker = ""
        if vl < best_val:
            best_val, best_state = vl, {
                k: v.cpu().clone() for k, v in model.state_dict().items()
            }
            marker = " *"
        print(
            f"Epoch {epoch+1}/{epochs}  train: {total_loss/count:.6f}  val: {vl:.6f}{marker}"
        )
    model.cpu().load_state_dict(best_state)
    torch.save(best_state, "nnue_best.pt")
    print(f"Best val loss: {best_val:.6f}")
    return model


# def train_nnue(data_path, epochs=30, lr=1e-3, bs=16384, load_model_path=None):
#     device = torch.device('cuda' if torch.cuda.is_available() else 'cpu')
#     print(f"Using {device}")
#     ds = NNUEData(data_path)
#     print(f"loaded {len(ds)} positions")
#     dl = DataLoader(ds, batch_size=bs, shuffle=True, num_workers=4, pin_memory=True)
#     model = NNUE().to(device)
#     if load_model_path:
#         print("loading model:", load_model_path)
#         model.load_state_dict(torch.load(load_model_path))
#     opt = torch.optim.Adam(model.parameters(), lr=lr)
#     loss_fn = nn.MSELoss()
#     for epoch in range(epochs):
#         start = time.time()
#         total_loss, n, t_data, t_gpu = 0.0, 0, 0.0, 0.0
#         t0 = time.time()
#         for x, y in dl:
#             t1 = time.time()
#             t_data += t1 - t0
#             x, y = x.to(device, non_blocking=True), y.to(device, non_blocking=True)
#             loss = loss_fn(model(x), y)
#             opt.zero_grad()
#             loss.backward()
#             opt.step()
#             t2 = time.time()
#             t_gpu += t2 - t1
#             total_loss += loss.item() * len(x)
#             n += len(x)
#             t0 = t2
#         end = time.time()
#         print(f"Epoch {epoch+1}/{epochs}  loss: {total_loss/n:.6f}  duration s: {end-start:.2f}   data: {t_data:.1f}s  gpu: {t_gpu:.1f}s")
#         torch.save(model.state_dict(), f"epoch_{epoch+1}_loss_{total_loss/n:.6f}nnue.pt")
#     return model.cpu()


# def train_nnue(data_path, epochs=10, lr=1e-3, bs=4096, load_model_path=None):
#     ds = NNUEData(data_path)
#     print(f"loaded {len(ds)} positions")
#     dl = DataLoader(ds, batch_size=bs, shuffle=True, num_workers=2)
#     model = NNUE()
#     if load_model_path:
#         print("loading model:", load_model_path)
#         model.load_state_dict(torch.load(load_model_path))
#     opt = torch.optim.Adam(model.parameters(), lr=lr)
#     loss_fn = nn.MSELoss()

#     for epoch in range(epochs):
#         start = time.time()
#         total_loss, n = 0.0, 0
#         for x, y in dl:
#             pred = model(x)
#             loss = loss_fn(pred, y)
#             opt.zero_grad()
#             loss.backward()
#             opt.step()
#             total_loss += loss.item() * len(x)
#             n += len(x)
#         end = time.time()
#         print(f"Epoch {epoch+1}/{epochs}  loss: {total_loss/n:.6f}  duration s: {end-start:.2f}")
#         torch.save(model.state_dict(), f"epoch_{epoch+1}_loss_{total_loss/n:.6f}nnue.pt")
#     return model


def export_weights(model, path="nnue.bin"):
    "Export NNUE weights to flat binary file for Rust inference"
    with open(path, "wb") as f:
        for name in ["l1", "l2", "l3"]:
            layer = getattr(model, name)
            f.write(layer.weight.data.numpy().tobytes())
            f.write(layer.bias.data.numpy().tobytes())


# def export_weights(model, path="nnue.bin"):
#     "Export NNUE weights to flat binary file for Rust inference"
#     with open(path, "wb") as f:
#         for name in ["l1", "l2", "l3"]:
#             layer = getattr(model, name)
#             f.write(layer.weight.data.numpy().tobytes())
#             f.write(layer.bias.data.numpy().tobytes())


if __name__ == "__main__":
    # print(fen_to_features("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"))
    # print(len(NNUEData("quiet-labeled.v7.epd")))
    # precompute_sparse("lichess-big3-resolved.book", "nnue_data.npz")

    indices, results = load_data("nnue_data.npz")

    model = train_nnue(
        indices,
        results,
        epochs=100,
        lr=5e-4
        # load_model_path="epoch_5_loss_0.069362nnue.pt",
    )
    #torch.save(model.state_dict(), "final_nnue.pt")

    model = NNUE()
    model.load_state_dict(torch.load("nnue_best.pt"))
    export_weights(model)

    # def eval_fen(model, fen):
    #     feats = fen_to_features(fen)
    #     x = torch.zeros(768)
    #     x[feats] = 1.0
    #     with torch.no_grad():
    #         raw = model.l3(torch.clamp(model.l2(torch.clamp(model.l1(x), 0, 1)), 0, 1))
    #     return raw.item()

    # eval = eval_fen(model, "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -")
    # print(eval)
