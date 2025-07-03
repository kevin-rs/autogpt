import matplotlib.pyplot as plt
import statistics as stats

with open("iac_benchmark.csv") as f:
    latencies = [int(line) for line in f if line.strip()]

plt.figure(figsize=(12, 6))
plt.plot(latencies, color='red', linewidth=1.2, label='Latency')
plt.axhline(stats.mean(latencies), color='blue', linestyle='--', linewidth=1, label='Mean')
plt.axhline(stats.median(latencies), color='green', linestyle='-.', linewidth=1, label='Median')

plt.title("Latency per Iteration (µs)", fontsize=16)
plt.xlabel("Iteration", fontsize=12)
plt.ylabel("Latency (µs)", fontsize=12)
plt.legend()
plt.grid(True, linestyle='--', alpha=0.6)
plt.tight_layout()
plt.show()
