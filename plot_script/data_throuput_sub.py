import pandas as pd
import matplotlib.pyplot as plt
import os
import argparse

def strip_name(name):
    return name.replace("log_", "").replace(".csv", "")

def plot_bar_chart(csv_file, name, ax):
    # Read the CSV file into a DataFrame
    df = pd.read_csv(csv_file)

    labels = ["total_data_sent", "total_data_received"]
    values = df[labels].div(df["time_us"], axis=0).mean()

    # Plotting the bar chart
    ax.bar(labels, values, width=0.4, color='blue', align='center')
    ax.set_title(f"Node {strip_name(name)}", fontsize=25)

def main():
    parser = argparse.ArgumentParser(description="Print the contents of all files in a directory.")
    parser.add_argument("directory", help="Path to directory")
    args = parser.parse_args()

    directory = args.directory
    if not os.path.exists(directory):
        print(f"Error: Directory '{directory}' does not exist.")
        return

    plt.rcParams.update({'font.size': 20})
    for root, _, files in os.walk(directory):
        fig, axs = plt.subplots(1, len(files), sharey=True)
        fig.suptitle('Data Throughput')
        axs[0].set_ylabel("byte per sec")

        for count, file in enumerate(files):
            csv_file = os.path.join(root, file)
            try:
                plot_bar_chart(csv_file, file, axs[count])
            except FileNotFoundError:
                print(f"Error: File '{csv_file}' not found.")
            except pd.errors.EmptyDataError:
                print(f"Error: File '{csv_file}' is empty or not in the expected CSV format.")
    plt.show()

if __name__ == "__main__":
    main()
