from typing import Callable
from matplotlib.dates import num2timedelta
import matplotlib.pyplot as plt
from matplotlib.ticker import FuncFormatter
import pandas as pd
import numpy as np

from pathlib import Path
from plot_config import PlotConfig, PlotStyle


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

    def timedelta_seconds_formatter() -> Callable:
        return FuncFormatter(
            lambda v, _: f"{pd.Timedelta(v, unit='ns').total_seconds():.0f}"
        )

    def percent_formatter() -> Callable:
        return FuncFormatter(lambda v, _: f"{v * 100.0:.0f}%")

    def target_throughput_extract_kB(v) -> float:
        return int(v[v.find("(") + 1 : v.find(")")]) / 1000

    def kB_formatter() -> Callable:
        return FuncFormatter(lambda v, _: f"{v / 1000.0:g}")

    def plot_rx_throughput_over_time(self):
        fig, ax = plt.subplots()

        df = self.df[["time_us", "data_received"]]
        # TODO: Check if we want origin="start" or not
        df_sec = df.rolling("1s", on="time_us").sum()

        ax.plot(df_sec["time_us"], df_sec["data_received"])

        ax.xaxis.set_major_formatter(Plotter.timedelta_seconds_formatter())
        ax.set_xlabel("Time [s]")
        ax.set_ylabel("RX Throughput [B]")

        self.post_function(f"rx_throughput_over_time_{self.label}")

    def plot_rx_sniff_throughput_over_time(self):
        fig, ax = plt.subplots()

        df = self.df[["time_us", "data_sniffed"]]
        # TODO: Check if we want origin="start" or not
        df_sec = df.rolling("1s", on="time_us").sum()

        ax.plot(df_sec["time_us"], df_sec["data_sniffed"])

        ax.xaxis.set_major_formatter(Plotter.timedelta_seconds_formatter())
        ax.set_xlabel("Time [s]")
        ax.set_ylabel("RX Throughput [B]")

        self.post_function(f"rx_sniff_throughput_over_time_{self.label}")

    def plot_tx_throughput_over_time(self):
        fig, ax = plt.subplots()

        df = self.df[["time_us", "data_sent"]]
        # TODO: Check if we want origin="start" or not
        df_sec = df.rolling("1s", on="time_us").sum()

        ax.plot(df_sec["time_us"], df_sec["data_sent"])

        ax.xaxis.set_major_formatter(Plotter.timedelta_seconds_formatter())
        ax.set_xlabel("Time [s]")
        ax.set_ylabel("TX Throughput [B]")

        self.post_function(f"tx_throughput_over_time_{self.label}")

    def plot_percent_decoded_over_time(self):
        fig, ax = plt.subplots()

        df = self.df[["time_us", "coded_received", "decoded_received"]]
        # TODO: Check if we want origin="start" or not
        df_sec = df.resample("1s", on="time_us", origin="start").sum()

        ax.plot(
            df_sec.index,
            (
                df_sec["decoded_received"]
                / (df_sec["coded_received"] + df_sec["decoded_received"])
            ).fillna(0),
        )

        ax.set_ylim(0.0, 1.0 + 0.1)

        ax.xaxis.set_major_formatter(Plotter.timedelta_seconds_formatter())
        ax.yaxis.set_major_formatter(Plotter.percent_formatter())

        ax.set_xlabel("Time [s]")
        ax.set_ylabel("Decoded %")

        self.post_function(f"percent_decoded_over_time_{self.label}")

    def plot_coding_gain_by_target_throughput(self, coding_df, nocoding_df):
        tgs_coding = coding_df["traffic_generator"].unique()
        tgs_nocoding = nocoding_df["traffic_generator"].unique()
        tgs = sorted(
            list(set(tgs_coding) & set(tgs_nocoding) - set(["None"])),
            key=Plotter.target_throughput_extract_kB,
        )

        coding_gains = []
        for tg in tgs:
            slice_coding = coding_df[coding_df["traffic_generator"] == tg]
            slice_nocoding = nocoding_df[nocoding_df["traffic_generator"] == tg]

            # NOTE: We need to normalize by the experiment duration,
            # since no two experiment run for the same exact time.
            pps_coding = (
                slice_coding["packets_sent"].sum()
                / slice_coding["time_us"].max().total_seconds()
            )
            pps_nocoding = (
                slice_nocoding["packets_sent"].sum()
                / slice_nocoding["time_us"].max().total_seconds()
            )

            coding_gains.append(pps_nocoding / pps_coding)

        fig, ax = plt.subplots()

        ax.plot(tgs, coding_gains, linestyle="--", marker="o")
        ax.set_xlabel("Target Throughput [kB]")
        ax.set_ylabel("Coding Gain")

        # NOTE: matplotlib does not allow str arguments for FuncFormatter for some reason,
        # so we have to use this workaround.
        xlabels = [f"{Plotter.target_throughput_extract_kB(v):g}" for v in tgs]
        ax.set_xticklabels(xlabels)

        self.post_function(f"coding_gain_by_target_throughput_{self.label}")

    def plot_percent_decoded_by_target_throughput(self, coding_df):
        tgs = list(coding_df["traffic_generator"].unique())
        if "None" in tgs:
            tgs.remove("None")
        tgs = sorted(
            tgs,
            key=Plotter.target_throughput_extract_kB,
        )
        if "None" in tgs:
            tgs.remove("None")

        percent_decodeds = []
        for tg in tgs:
            slice_coding = coding_df[coding_df["traffic_generator"] == tg]

            native_packets = slice_coding["natives_received"].sum()
            decoded_packets = slice_coding["decoded_received"].sum()
            coded_packets = slice_coding["coded_received"].sum()

            percent_decodeds.append(
                (decoded_packets + native_packets)
                / (coded_packets + decoded_packets + native_packets)
            )

        fig, ax = plt.subplots()

        ax.plot(tgs, percent_decodeds, linestyle="--", marker="o")
        ax.set_xlabel("Target Throughput [kB]")
        ax.set_ylabel("Decoded %")

        # NOTE: matplotlib does not allow str arguments for FuncFormatter for some reason,
        # so we have to use this workaround.
        xlabels = [f"{Plotter.target_throughput_extract_kB(v):g}" for v in tgs]
        ax.set_xticklabels(xlabels)
        ax.yaxis.set_major_formatter(Plotter.percent_formatter())

        self.post_function(f"percent_decoded_by_target_throughput_{self.label}")

    def plot_target_throughput_vs_achieved_throughput(self, coding_df, nocoding_df):
        tgs_coding = coding_df["traffic_generator"].unique()
        tgs_nocoding = nocoding_df["traffic_generator"].unique()
        tgs = sorted(
            list(set(tgs_coding) & set(tgs_nocoding) - set(["None"])),
            key=Plotter.target_throughput_extract_kB,
        )

        achieved_coding = []
        achieved_nocoding = []
        for tg in tgs:
            slice_coding = coding_df[coding_df["traffic_generator"] == tg]
            slice_nocoding = nocoding_df[nocoding_df["traffic_generator"] == tg]

            throughput_coding = (
                slice_coding["data_sent"].sum()
                / slice_coding["time_us"].max().total_seconds()
            )
            throughput_nocoding = (
                slice_nocoding["data_sent"].sum()
                / slice_nocoding["time_us"].max().total_seconds()
            )

            achieved_coding.append(throughput_coding)
            achieved_nocoding.append(throughput_nocoding)

        fig, ax = plt.subplots()

        ax.plot(tgs, achieved_coding, linestyle="--", marker="o", label="Coding")
        ax.plot(tgs, achieved_nocoding, linestyle="--", marker="o", label="No Coding")
        ax.set_xlabel("Target Throughput [kB]")
        ax.set_ylabel("Total Achieved Throughput [kB]")

        # NOTE: matplotlib does not allow str arguments for FuncFormatter for some reason,
        # so we have to use this workaround.
        xlabels = [f"{Plotter.target_throughput_extract_kB(v):g}" for v in tgs]
        ax.set_xticklabels(xlabels)
        ax.yaxis.set_major_formatter(Plotter.kB_formatter())

        ax.legend()

        self.post_function(f"target_throughput_vs_achieved_throughput_{self.label}")

    # FIXME: This does not work correctly at all, needs to be fixed
    # I think we just have to combine all stored files inside the
    # log directory and then call the plot function.
    def plot_rx_tx_barchart(self):
        fig, ax = plt.subplots()

        nodes = self.df["node_id"].unique()
        df = self.df[
            (self.df["time_us"] >= pd.Timedelta(10, unit="s"))
            & (
                self.df["time_us"]
                <= self.df.iloc[-1]["time_us"] - pd.Timedelta(5, unit="s")
            )
        ]
        throughputs = {
            "data_sent": [],
            "data_received": [],
        }
        secs = df.iloc[-1]["time_us"].seconds

        for n in nodes:
            for l in ["data_received", "data_sent"]:
                tp = df[df["node_id"] == n][l].sum() / secs
                throughputs[l].append(tp)

        x = np.arange(len(nodes))  # the label locations
        width = 0.25  # the width of the bars
        multiplier = 0

        for attribute, measurement in throughputs.items():
            offset = width * multiplier
            rects = ax.bar(x + offset, measurement, width, label=attribute)
            ax.bar_label(rects, padding=3)
            multiplier += 1

        ax.set_ylabel("Throughput [B/s]")
        ax.set_xticks(x + width, [f"Node {n}" for n in nodes])
        ax.legend()

        self.post_function(f"rx_tx_barchart_{self.label}")

    # TODO: Plots for:
    # Cache Efficiency (Hits vs Drops?)
    # Error Rates (TX and RX)
    # Coding vs. Non-Coding (Throughput)
    # targeted vs. achieved troughput (for TrafficGenerators that set targeted throughput)
