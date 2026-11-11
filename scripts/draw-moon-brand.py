"""Prepare the runtime icon, multi-resolution ICO and login lockup from the supplied PNG."""

from pathlib import Path

from PIL import Image

BRANDING = Path(__file__).resolve().parents[1] / "assets" / "branding"
SIZES = [16, 20, 24, 32, 40, 48, 64, 128, 256]


def main():
    with Image.open(BRANDING / "dragon-viewer-brand-source.png") as source:
        assert source.mode == "RGBA" and source.size == (1536, 1024)
        # Keep the named lockup together and strip only its transparent margins.
        source.crop((65, 4, 1471, 990)).save(BRANDING / "dragon-viewer-lockup.png")
        # Crop the emblem above the text and fit it to the Windows icon canvas.
        badge = source.crop((362, 7, 1173, 697))
    badge.thumbnail((248, 248), Image.Resampling.LANCZOS)
    icon = Image.new("RGBA", (256, 256))
    icon.alpha_composite(badge, ((256 - badge.width) // 2, (256 - badge.height) // 2))
    icon.save(BRANDING / "dragon-viewer-icon.png")
    icon.save(BRANDING / "dragon-viewer.ico", sizes=[(size, size) for size in SIZES], bitmap_format="bmp")


if __name__ == "__main__":
    main()
