import numpy as np

data = np.fromfile('nnue/nnue.bin', dtype=np.float32)

w1 = data[:512*768].reshape(512, 768)
b1 = data[512*768 : 512*768 + 512]
w2 = data[512*768 + 512 : 512*768 + 512 + 32*512].reshape(32, 512)
b2 = data[512*768 + 512 + 32*512 : 512*768 + 512 + 32*512 + 32]
w3 = data[512*768 + 512 + 32*512 + 32 : 512*768 + 512 + 32*512 + 32 + 32]
b3 = data[-1]

QA = 256  # quantization factor for activations/weights

w1_q = np.round(w1 * QA).clip(-32768, 32767).astype(np.int16)
b1_q = np.round(b1 * QA).clip(-32768, 32767).astype(np.int16)
w2_q = np.round(w2 * QA).clip(-32768, 32767).astype(np.int16)
b2_q = np.round(b2 * QA * QA).astype(np.int32)  # scale = QA²
# w3 and b3 stay as f32
w3_f = w3.astype(np.float32)
b3_f = np.array([b3], dtype=np.float32)

with open('nnue_q.bin', 'wb') as f:
    f.write(w1_q.tobytes())   # 512*768 i16
    f.write(b1_q.tobytes())   # 512 i16
    f.write(w2_q.tobytes())   # 32*512 i16
    f.write(b2_q.tobytes())   # 32 i32
    f.write(w3_f.tobytes())   # 32 f32
    f.write(b3_f.tobytes())   # 1 f32

print(f"Written nnue_q.bin: {w1_q.nbytes + b1_q.nbytes + w2_q.nbytes + b2_q.nbytes + w3_f.nbytes + b3_f.nbytes} bytes")
