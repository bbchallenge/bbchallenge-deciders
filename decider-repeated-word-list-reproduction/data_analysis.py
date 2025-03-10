import matplotlib.pyplot as plt
import tqdm
import numpy as np
from matplotlib.colors import LogNorm

FILE_MACHINES_LIST = "Coq-BB5_RWL_parameters_bbchallenge_format.txt"
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

# Create a colormap-based scatter plot with log normalization
scatter = plt.scatter(
    unique_B,
    unique_T,
    c=counts,
    cmap="viridis",  # A perceptually uniform colormap
    s=100,  # Fixed size for all points
    alpha=1,
    edgecolors="k",  # Black edge for better visibility
    linewidths=0.5,
    norm=norm,  # Apply logarithmic normalization
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
    "RepWL parameters pairs for the 6,577 machines decided by RepWL in Coq-BB5\n(Note: several machines use the same parameters)",
    fontsize=14,
)

# Set x-axis to start at 0
plt.xlim(left=0)

# Add grid for better readability with discrete y-axis
plt.grid(True, axis="y", linestyle="--", alpha=0.7)

plt.tight_layout()
plt.savefig("repwl_parameters_pairs_log_scale.pdf", format="pdf", bbox_inches="tight")

plt.show()
