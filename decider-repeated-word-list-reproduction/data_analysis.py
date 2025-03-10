import matplotlib.pyplot as plt
import tqdm
import numpy as np
from matplotlib.colors import LogNorm
import os
import pickle
from matplotlib.ticker import FuncFormatter

from decider import deciderRep_WL

FILE_MACHINES_LIST = "Coq-BB5_RWL_parameters_bbchallenge_format.txt"
PARAMS_CACHE_FILE = "params_pairs_count.pkl"
NODE_COUNTS_CACHE_FILE = "node_counts.pkl"

# Check if the cached params data exists
if os.path.exists(PARAMS_CACHE_FILE):
    print(f"Loading cached parameter counts from {PARAMS_CACHE_FILE}")
    with open(PARAMS_CACHE_FILE, "rb") as f:
        params_pairs_count = pickle.load(f)

    # Reconstruct params_B and params_T from the cached data
    params_B = []
    params_T = []
    for (b, t), count in params_pairs_count.items():
        # Add each (b,t) pair to the lists count times
        params_B.extend([b] * count)
        params_T.extend([t] * count)
else:
    print(f"Computing parameter counts from {FILE_MACHINES_LIST}")
    with open(FILE_MACHINES_LIST) as f:
        file_content = f.read()

    params_B = []
    params_T = []
    params_pairs_count = {}

    for line in tqdm.tqdm(file_content.split("\n")):
        if line.strip() == "":
            continue
        TM, BLOCK_SIZE, PLUS_THRESHOLD = line.split(" ")
        params_B.append(int(BLOCK_SIZE))
        params_T.append(int(PLUS_THRESHOLD))
        if (int(BLOCK_SIZE), int(PLUS_THRESHOLD)) not in params_pairs_count:
            params_pairs_count[(int(BLOCK_SIZE), int(PLUS_THRESHOLD))] = 0
        params_pairs_count[(int(BLOCK_SIZE), int(PLUS_THRESHOLD))] += 1

    # Save the computed data
    print(f"Saving parameter counts to {PARAMS_CACHE_FILE}")
    with open(PARAMS_CACHE_FILE, "wb") as f:
        pickle.dump(params_pairs_count, f)

# Check if node_counts is already computed and cached
if os.path.exists(NODE_COUNTS_CACHE_FILE):
    print(f"Loading cached node counts from {NODE_COUNTS_CACHE_FILE}")
    with open(NODE_COUNTS_CACHE_FILE, "rb") as f:
        node_counts = pickle.load(f)
else:
    print("Computing node counts...")
    # This is where you would compute node_counts from your data
    # For example, if you're tracking the number of nodes in some structure:
    node_counts = []

    with open(FILE_MACHINES_LIST) as f:
        file_content = f.read()

    for line in tqdm.tqdm(file_content.split("\n")):
        if line.strip() == "":
            continue
        TM, BLOCK_SIZE, PLUS_THRESHOLD = line.split(" ")

        # This is a placeholder for your actual node count calculation
        # Replace this with your actual logic to calculate node_count
        success, reason_failure, node_count, has_regex_branching = deciderRep_WL(
            TM,
            int(BLOCK_SIZE),
            int(PLUS_THRESHOLD),
            150001,
            451,
            False,
            False,
            True,
            "",
            False,
            False,
        )

        if (
            node_count > 140000
            or node_count == 42
            or (node_count >= 1200 and node_count <= 1300)
        ):
            print(f"Node count: {node_count} for {TM} {BLOCK_SIZE} {PLUS_THRESHOLD}")

        node_counts.append(node_count)

    # Save the computed node counts
    print(f"Saving node counts to {NODE_COUNTS_CACHE_FILE}")
    with open(NODE_COUNTS_CACHE_FILE, "wb") as f:
        pickle.dump(node_counts, f)

# Enable full LaTeX rendering
plt.rcParams["text.usetex"] = True
plt.rcParams["font.family"] = "serif"  # Use LaTeX's default font
plt.rcParams["mathtext.fontset"] = "cm"  # Use LaTeX's default font
plt.rcParams["text.latex.preamble"] = (
    r"\usepackage{amsmath,amssymb}"  # Load math symbols
)


# Create a new figure with the updated approach
plt.figure(figsize=(10, 6))

# Create a dictionary to store unique (B,T) pairs and their counts
unique_points = {}
for b, t in zip(params_B, params_T):
    if (b, t) not in unique_points:
        unique_points[(b, t)] = params_pairs_count[(b, t)]

# Extract unique coordinates and their counts
unique_B = [point[0] for point in unique_points.keys()]
unique_T = [point[1] for point in unique_points.keys()]
counts = list(unique_points.values())

# Apply logarithmic normalization to better differentiate small values
norm = LogNorm(vmin=min(counts), vmax=max(counts))

# Add grid for better readability with discrete y-axis
plt.grid(True, axis="y", linestyle="--", alpha=0.7, zorder=0)  # Lower zorder for grid

# Create a colormap-based scatter plot with log normalization
scatter = plt.scatter(
    unique_B,
    unique_T,
    c=counts,
    cmap="viridis",  # A perceptually uniform colormap
    s=100,  # Fixed size for all points
    alpha=0.9,
    edgecolors="k",  # Black edge for better visibility
    linewidths=0.5,
    norm=norm,  # Apply logarithmic normalization
    zorder=10,  # Higher zorder to ensure points are on top
)

# Add a colorbar to show the frequency scale
cbar = plt.colorbar(scatter)
cbar.set_label("Number of machines (log scale)", rotation=270, labelpad=20, fontsize=14)

# Make y-axis discrete by setting integer ticks
y_unique = sorted(list(set(unique_T)))
plt.yticks(y_unique)

# Ensure x-axis shows tick for the biggest point
max_x = max(unique_B)
plt.xticks(list(plt.xticks()[0]) + [max_x])

# Add labels and title
plt.xlabel("Block length parameter $l$", fontsize=14)
plt.ylabel("Block repeat threshold parameter $T$", fontsize=14)
# plt.title(
#     r"RepWL parameters pairs for the 6,577 machines decided by RepWL in Coq-BB5\n $\small{\texit{Note}\text{ several machines use the same parameters}}$"
# )

plt.title(
    "RepWL parameters pairs and frequency for the 6,577 machines decided by RepWL in Coq-BB5",
    fontsize=14,
)

# Set x-axis to start at 0
plt.xlim(left=0)

plt.tight_layout()
plt.savefig("repwl_parameters_pairs_log_scale.pdf", format="pdf", bbox_inches="tight")
plt.show()

# Create a histogram of the node counts
plt.figure(figsize=(10, 6))

# Enable LaTeX rendering for the histogram
plt.rcParams["text.usetex"] = True
plt.rcParams["font.family"] = "serif"
plt.rcParams["mathtext.fontset"] = "cm"
plt.rcParams["text.latex.preamble"] = r"\usepackage{amsmath,amssymb}"

# Create histogram with logarithmic y-scale
plt.hist(node_counts, bins=30, color="darkred", edgecolor="black", alpha=0.8)
plt.yscale("log")  # Use log scale for y-axis to better show distribution

# Add labels and title
plt.xlabel("RepWL graph node count per machine", fontsize=14)
plt.ylabel("Frequency (log scale)", fontsize=14)
plt.title(
    "Distribution of node counts in the 6,577 RepWL graphs constructed by Coq-BB5",
    fontsize=14,
)


# Format x-axis with thousands separators
def thousands_formatter(x, pos):
    return f"{int(x):,}"


plt.gca().xaxis.set_major_formatter(FuncFormatter(thousands_formatter))

# Add grid for better readability
plt.grid(True, axis="y", linestyle="--", alpha=0.7, zorder=0)

# Add some statistics as text with thousands separators
max_count = max(node_counts)
min_count = min(node_counts)
mean_count = np.mean(node_counts)
median_count = np.median(node_counts)

stats_text = (
    f"Max: {max_count:,}\n"
    f"Min: {min_count:,}\n"
    f"Mean: {int(mean_count):,}\n"
    f"Median: {int(median_count):,}"
)

# Position the text box in the upper right corner
plt.annotate(
    stats_text,
    xy=(0.95, 0.95),
    xycoords="axes fraction",
    bbox=dict(boxstyle="round,pad=0.5", facecolor="white", alpha=0.8),
    ha="right",
    va="top",
    fontsize=12,
)

plt.tight_layout()
plt.savefig("repwl_node_counts_histogram.pdf", format="pdf", bbox_inches="tight")
plt.show()
