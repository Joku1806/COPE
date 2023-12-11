import pandas as pd
import matplotlib.pyplot as plt
import os
import argparse

def plot_bar_chart(csv_file):
    # Read the CSV file into a DataFrame
    df = pd.read_csv(csv_file)

    labels = ["total_data_send", "total_data_rec"]
    values = df[labels].div(df["time"], axis=0).mean()

    # Plotting the bar chart
    plt.bar(labels, values, width=0.1, color='blue')
    plt.ylabel('Throughput in bytes per sec')
    plt.title('Data Throughput (%s)' % (csv_file))
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
