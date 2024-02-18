import pandas as pd

from typing import List


class DataReader:
    filepaths: List[str]

    def __init__(
        self,
        filepaths: List[str],
    ):
        self.filepaths = filepaths

    def undo_wraparounds(column: pd.Series) -> pd.Series:
        wrap_indices = column.index[column.shift(1) > column]
        column = column.to_frame(0).apply(
            lambda v: v + len(wrap_indices[v.name >= wrap_indices]) * 0xFFFFFFFF,
            axis=1,
        )[0]

        return column

    def subtract_previous_value(column: pd.Series) -> pd.Series:
        return column - column.shift(1, fill_value=0)

    def read_single_file(self, path: str) -> pd.DataFrame:
        df = pd.read_csv(path, converters={"traffic_generator": str})

        cols = df.columns.difference(
            ["time_us", "node_id", "own_mac", "target_id", "traffic_generator"]
        )
        df.loc[:, cols] = df.loc[:, cols].apply(
            DataReader.undo_wraparounds, axis="index"
        )
        df.loc[:, cols] = df.loc[:, cols].apply(
            DataReader.subtract_previous_value, axis="index"
        )
        df["time_us"] = pd.to_timedelta(df["time_us"], unit="us")
        df["time_us"] -= df["time_us"].iloc[0]

        return df

    def read(self) -> pd.DataFrame:
        dfs = [self.read_single_file(path) for path in self.filepaths]
        combined = pd.concat(dfs)
        combined.sort_values("time_us", inplace=True)
        return combined
