import matplotlib.pyplot as plt
from enum import Enum, auto


class PlotStyle(Enum):
    Default = (auto(),)
    Presentation = (auto(),)


class PlotConfig:
    def __init__(self):
        pass

    def get_preset_for_style(style: PlotStyle):
        if style == PlotStyle.Default:
            return {
                "figure.autolayout": True,
                "savefig.dpi": 300,
                "savefig.format": "png",
            }

        if style == PlotStyle.Presentation:
            # NOTE: Seaborn Preset changes all specific font parameters
            # instead of inheriting from font.size, so we have to
            # change all of them.
            base_font_size = 12
            return {
                "figure.autolayout": True,
                "figure.figsize": (9.5, 4.25),
                "axes.labelsize": base_font_size * 1.1,
                "axes.titlesize": base_font_size * 1.2,
                "legend.fontsize": base_font_size,
                "xtick.labelsize": base_font_size,
                "ytick.labelsize": base_font_size,
                "font.size": base_font_size,
                "font.family": "monospace",
                "font.monospace": ["IBM Plex Mono"],
                "savefig.dpi": 300,
                "savefig.format": "png",
            }

    def set_style(style: PlotStyle):
        if style == PlotStyle.Default:
            plt.style.use("seaborn-v0_8-pastel")
        elif style == PlotStyle.Paper:
            plt.style.use("seaborn-v0_8-paper")
        elif style == PlotStyle.Poster:
            plt.style.use("seaborn-v0_8-poster")
        elif style == PlotStyle.Presentation:
            plt.style.use("seaborn-v0_8-paper")

        plt.rcParams.update(PlotConfig.get_preset_for_style(style))
