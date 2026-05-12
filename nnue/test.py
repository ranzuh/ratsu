import numpy as np

data = np.fromfile('nnue/nnue.bin', dtype=np.float32)

# Layout: w1[512][768], b1[512], w2[32][512], b2[32], w3[32], b3[1]
w1 = data[:512*768].reshape(512, 768)
b1 = data[512*768 : 512*768 + 512]
w2 = data[512*768 + 512 : 512*768 + 512 + 32*512].reshape(32, 512)
b2 = data[512*768 + 512 + 32*512 : 512*768 + 512 + 32*512 + 32]
w3 = data[512*768 + 512 + 32*512 + 32 : 512*768 + 512 + 32*512 + 32 + 32]
b3 = data[-1]

for name, arr in [('w1', w1), ('b1', b1), ('w2', w2), ('b2', b2), ('w3', w3), ('b3', b3)]:
    print(f"{name:3s}  min={arr.min():.4f}  max={arr.max():.4f}  absmax={np.abs(arr).max():.4f}")