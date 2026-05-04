import time
import ratsu

import json


def save_weights(weights, filename):
    with open(filename, "w") as f:
        json.dump(weights, f)


def load_weights(filename):
    with open(filename) as f:
        return json.load(f)


def load_positions(filename):
    positions = []
    with open(filename) as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            # fen, result = line.rsplit('[', 1)
            # result = float(result.rstrip(']'))
            fen, epd_result = line.split(' "', 1)
            epd_result = epd_result.rstrip('";')
            if epd_result == "1-0":
                result = 1.0
            elif epd_result == "0-1":
                result = 0.0
            elif epd_result == "1/2-1/2":
                result = 0.5
            else:
                raise ValueError(f"Invalid result: {epd_result}")
            positions.append((fen.strip(), result))
    return positions


def mse(weights, positions, K=1.13):
    error = 0.0
    for fen, result in positions:
        score = ratsu.eval_fen(fen, weights)
        sigmoid = 1 / (1 + 10 ** (-K * score / 400))
        error += (result - sigmoid) ** 2
    return error / len(positions)


MATERIAL_INDICES = {0, 1, 2, 3, 4, 5}
BONUS_INDICES = {6, 7, 8, 9, 10, 11, 12, 13}
PST_INDICES = [(14 + i * 64, 14 + i * 64 + 64) for i in range(6)]
PAWN_PST_INDICES = {i for i in range(14, 14 + 64)}

IMPROVED_ERROR_TOLERANCE = 0.000001

def tune_with_annealing(dataset, initial_weights, K=1.13):
    best_weights = initial_weights.copy()
    best_error = ratsu.compute_mse(dataset, best_weights, K)
    print(f"Starting MSE: {best_error}")

    step_sizes = [30, 10, 5, 2, 1]  # coarse to fine

    for step in step_sizes:
        print(f"\n--- Step size: {step} ---")
        improved = True
        while improved:
            improved = False
            # TODO: shuffle parameter order to avoid bias
            for i in range(len(best_weights)):
                if i in MATERIAL_INDICES:
                    continue  # skip material weights for now
                # if i not in BONUS_INDICES:
                #     continue  # only tune piece-square weights for now
                # if i not in PAWN_PST_INDICES:
                #     continue  # only tune pawn PST for now
                start_time = time.time()
                new_weights = best_weights.copy()
                new_weights[i] += step
                new_error = ratsu.compute_mse(dataset, new_weights, K)
                if new_error < best_error - IMPROVED_ERROR_TOLERANCE:
                    assert i not in {14, 15, 16, 17, 18, 19, 20, 21}
                    best_error = new_error
                    best_weights = new_weights
                    improved = True
                else:
                    new_weights[i] -= 2 * step
                    new_error = ratsu.compute_mse(dataset, new_weights, K)
                    if new_error < best_error - IMPROVED_ERROR_TOLERANCE:
                        assert i not in {14, 15, 16, 17, 18, 19, 20, 21}
                        best_error = new_error
                        best_weights = new_weights
                        improved = True

                end_time = time.time()
                elapsed = end_time - start_time
                print(
                    f"param {i}/{len(best_weights)}, MSE: {best_error:.6f}, iter took: {elapsed:.2f}s"
                )

            save_weights(
                best_weights, f"tuner/weights_step{step}_mse{best_error:.6f}.json"
            )
            print(f"Pass complete, MSE: {best_error:.6f}, weights saved.")

    return best_weights


if __name__ == "__main__":
    positions = load_positions("tuner/quiet-labeled.v7.epd")
    print(f"Loaded {len(positions)} positions")

    dataset = ratsu.EvaluationDataset(positions)

    init_weights = load_weights("tuner/weights_material_only.json")
    print(f"loaded {len(init_weights)} weights")

    # debug stuff
    # set white pawn at c6 pst value manually
    # init_weights[32] = 10
    # set passed pawn bonus manually
    # init_weights[9] = 10
    # print("mse: ", ratsu.compute_mse(positions, init_weights, 1.13))

    # find optimal K for current weights
    # best_k, best_mse = 0, float('inf')
    # for k_int in range(140, 200, 1):
    #     print(f"Testing K={k_int/100:.2f}...")
    #     k = k_int / 100
    #     mse = ratsu.compute_mse(positions, init_weights, k)
    #     if mse < best_mse:
    #         print(f"New best K: {k}, MSE: {mse}")
    #         best_k, best_mse = k, mse
    # print(f"Optimal K: {best_k}, MSE: {best_mse}")

    best_k = 1.50

    tuned_weights = tune_with_annealing(dataset, init_weights, K=best_k)
    print("Tuned weights:", tuned_weights)
