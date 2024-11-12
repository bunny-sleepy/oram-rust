import numpy as np
import matplotlib.pyplot as plt

def load_data_from_file(filename):
    # Load data from the specified file
    with open(filename, 'r') as file:
        lines = file.readlines()
    
    # Parse the total number of accesses from the first line
    total_accesses = int(lines[0].split(',')[1])
    
    # Parse the rest of the file into a dictionary
    stash_data = {int(line.split(',')[0]): int(line.split(',')[1]) for line in lines[1:]}
    
    return total_accesses, stash_data

def plot_and_save_figure(total_accesses, stash_data, config_label):
    # Calculate δ(R) and log2(1/δ(R))
    
    R_values = sorted(stash_data.keys())
    delta_R = []
    log_delta_inv_R = []
    print(R_values)

    for R in R_values:
        # exceed_count = sum(count for stash_size, count in stash_data.items() if stash_size > R)
        delta_R_value = stash_data[R] / total_accesses
        delta_R.append(delta_R_value)
        log_delta_inv_R.append(np.log2(1 / delta_R_value) if delta_R_value > 0 else 0)

    # Plotting
    plt.figure(figsize=(10, 6))
    plt.plot(R_values, log_delta_inv_R, marker='o', linestyle='-')
    print(log_delta_inv_R)
    plt.xlabel("R (Stash Size)")
    plt.ylabel(r"$\log_2\left(\frac{1}{\delta(R)}\right)$")
    plt.title(f"Logarithmic Plot of Stash Size Exceedance Probability for {config_label}")
    plt.grid(True)
    
    # Save the plot as a PNG file
    output_filename = f"stash_plot_{config_label}.png"
    plt.savefig(output_filename)
    print(f"Plot saved as {output_filename}")
    plt.close()

# File names for different configurations
file_configs = {
    'Z2': '/home/wenhaowang/projects/oram-rust/stash_data_N1048576_Z2_B32.txt',
    'Z4': '/home/wenhaowang/projects/oram-rust/stash_data_N1048576_Z4_B32.txt',
    'Z6': '/home/wenhaowang/projects/oram-rust/stash_data_N1048576_Z6_B32.txt'
}

# Generate and save plots for each configuration
for config_label, filename in file_configs.items():
    total_accesses, stash_data = load_data_from_file(filename)
    plot_and_save_figure(total_accesses, stash_data, config_label)