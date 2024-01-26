import pandas as pd
import matplotlib.pyplot as plt
import os
import argparse

def plot_bar_chart(csv_file):
    # Read the CSV file into a DataFrame
    df = pd.read_csv(csv_file)

    labels = ["total_data_sent", "total_data_received"]
    # FIXME: This is broken right now, we need to convert
    # to some python native duration and grab seconds from that.
    values = df[labels].div(df["time_us"], axis=0).mean()

    # Plotting the bar chart
    plt.rcParams.update({'font.size': 30})
    plt.bar(labels, values, width=0.1, color='blue')
    plt.ylabel('Throughput in bytes per sec', fontsize=22)
    plt.title('Data Throughput (%s)' % (csv_file), fontsize=25)
    plt.show()

def main():
    parser = argparse.ArgumentParser(description="Print the contents of all files in a directory.")
    parser.add_argument("directory", help="Path to directory")
    args = parser.parse_args()

    directory = args.directory
    if not os.path.exists(directory):
        print(f"Error: Directory '{directory}' does not exist.")
        return

    for root, _, files in os.walk(directory):
        for file in files:
            csv_file = os.path.join(root, file)
            try:
                plot_bar_chart(csv_file)
            except FileNotFoundError:
                print(f"Error: File '{csv_file}' not found.")
            except pd.errors.EmptyDataError:
                print(f"Error: File '{csv_file}' is empty or not in the expected CSV format.")


if __name__ == "__main__":
    main()
