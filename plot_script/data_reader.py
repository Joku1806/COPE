import pandas as pd


class DataReader:
    filepath: str

    def __init__(
        self,
        filepath: str,
    ):
        self.filepath = filepath

    def undo_wraparounds(column: pd.Series) -> pd.Series:
        wrap_indices = column.index[column.shift(1) > column]
        column = column.to_frame(0).apply(
            lambda v: v + len(wrap_indices[v.name >= wrap_indices]) * 0xFFFFFFFF,
            axis=1,
        )[0]

        return column

    def subtract_previous_value(column: pd.Series) -> pd.Series:
        return column - column.shift(1, fill_value=0)

    def read(self) -> pd.DataFrame:
        df = pd.read_csv(self.filepath)
        df = df.apply(DataReader.undo_wraparounds, axis="index")
        df.loc[:, df.columns != "time_us"] = df.loc[:, df.columns != "time_us"].apply(
            DataReader.subtract_previous_value, axis="index"
        )
        df["time_us"] = pd.to_timedelta(df["time_us"], unit="us")

        return df
