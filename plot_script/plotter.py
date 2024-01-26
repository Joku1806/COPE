import matplotlib.pyplot as plt
import pandas as pd

from pathlib import Path
from .plot_config import PlotConfig, PlotStyle


class Plotter:
    df: pd.DataFrame
    result_directory: str
    label: str
    interactive: bool
    save: bool
    set_figure_title: bool

    def __init__(
        self,
        df: pd.DataFrame,
        result_directory: str,
        label: str,
        style: PlotStyle = PlotStyle.Default,
        interactive: bool = False,
        save: bool = True,
        set_figure_title: bool = False,
    ):
        self.df = df
        self.result_directory = result_directory
        self.label = label
        self.interactive = interactive
        self.save = save
        self.set_figure_title = set_figure_title
        PlotConfig.set_style(style)

    def save_plot(self, name: str):
        Path(self.result_directory).mkdir(parents=True, exist_ok=True)
        plt.savefig(
            f"{self.result_directory}/{name.replace(' ', '_').replace('.', '')}"
        )

    def post_function(self, filename):
        if self.set_figure_title:
            plt.title(self.label)

        if self.save:
            self.save_plot(filename)

        if self.interactive:
            plt.show()

        plt.close()

    def plot_rx_throughput_over_time(self):
        fig, ax = plt.figure()

        df = self.df[["time_us", "total_data_received"]]
        df_sec = df.resample("1s", on="time_us").sum()

        ax.plot(df_sec["time_us"], df_sec["total_data_received"])
        ax.set_xlabel("Time [s]")
        ax.set_ylabel("RX Throughput [B]")

        self.post_function(f"rx_throughput_over_time_{self.label}")

    def plot_rx_tx_barchart(self):
        fig, ax = plt.figure()

        labels = ["total_data_sent", "total_data_received"]

        ax.bar(
            labels,
            self.df[labels].sum(axis="columns") / self.df.iloc[-1]["time_us"].seconds,
        )
        ax.set_ylabel("Throughput [B/s]")

        self.post_function(f"rx_tx_barchart_{self.label}")

    # TODO: Plots for:
    # Cache Efficiency (Hits vs Drops?)
    # Error Rates (TX and RX)
    # others (read the paper again to find out what is interesting to look at)
