# /// script
# requires-python = ">=3.11"
# dependencies = ["pillow==12.1.0"]
# ///
"""Generate all project icons from the approved, unmodified source image."""

from pathlib import Path

from PIL import Image

root = Path(__file__).resolve().parent.parent
directory = root / "assets" / "branding"
web = root / "examples" / "website" / "public" / "brand"
with Image.open(directory / "solid-gpui.png") as source:
    image = source.convert("RGBA")
    if image.width != image.height:
        raise ValueError("The approved project icon must be square")
    for destination, sizes in ((directory, (64, 128, 256, 512)), (web, (32, 180))):
        destination.mkdir(parents=True, exist_ok=True)
        for size in sizes:
            image.resize((size, size), Image.Resampling.LANCZOS).save(
                destination / f"icon-{size}.png", optimize=True
            )
    image.save(
        directory / "solid-gpui.ico",
        sizes=[(size, size) for size in (16, 24, 32, 48, 64, 128, 256)],
    )
    image.save(directory / "solid-gpui.icns")

print(f"Generated PNG, ICO, and ICNS icons in {directory} and {web}")
