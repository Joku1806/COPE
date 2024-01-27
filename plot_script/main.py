import pandas as pd
import os
import argparse

from data_reader import DataReader
from plotter import Plotter


def strip_name(name):
    return name.replace("log_", "").replace(".csv", "")


def main():
    parser = argparse.ArgumentParser(
        description="Print the contents of all files in a directory."
    )
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
                df = DataReader(csv_file).read()
                plotter = Plotter(df, "results", f"Node {strip_name(file)}")

                plotter.plot_rx_throughput_over_time()
                plotter.plot_rx_tx_barchart()
            except FileNotFoundError:
                print(f"Error: File '{csv_file}' not found.")
            except pd.errors.EmptyDataError:
                print(
                    f"Error: File '{csv_file}' is empty or not in the expected CSV format."
                )


if __name__ == "__main__":
    main()
