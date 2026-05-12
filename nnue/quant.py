import numpy as np

data = np.fromfile('nnue-new.bin', dtype=np.float32)

# 768→256→1 layout
w1 = data[:256*768].reshape(256, 768)
b1 = data[256*768 : 256*768 + 256]
w2 = data[256*768 + 256 : 256*768 + 256 + 256]  # 256 weights (not 32x256)
b2 = data[256*768 + 256 + 256]                    # 1 scalar

QA = 256

w1_q = np.round(w1 * QA).clip(-32768, 32767).astype(np.int16)
b1_q = np.round(b1 * QA).clip(-32768, 32767).astype(np.int16)
# Output layer stays f32 (tiny, not worth quantizing)
w2_f = w2.astype(np.float32)
b2_f = np.array([b2], dtype=np.float32)

with open('nnue_q.bin', 'wb') as f:
    f.write(w1_q.tobytes())   # 256*768 i16
    f.write(b1_q.tobytes())   # 256 i16
    f.write(w2_f.tobytes())   # 256 f32
    f.write(b2_f.tobytes())   # 1 f32

print(f"Written nnue_q.bin")
