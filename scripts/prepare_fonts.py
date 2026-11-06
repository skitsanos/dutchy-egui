"""Prepare renamed static OFL fonts. Run with: uv run --with fonttools scripts/prepare_fonts.py"""

import hashlib
import io
import json
from pathlib import Path
from urllib.parse import quote
from urllib.request import urlopen

from fontTools import version
from fontTools.ttLib import TTFont
from fontTools.varLib.instancer import instantiateVariableFont

ROOT = Path(__file__).resolve().parents[1] / "assets" / "fonts"
FAMILIES = (
    ("inter", "Inter[opsz,wght].ttf", "Body"),
    ("spacegrotesk", "SpaceGrotesk[wght].ttf", "Display"),
    ("jetbrainsmono", "JetBrainsMono[wght].ttf", "Mono"),
)
WEIGHTS = ((400, "Regular"), (500, "Medium"), (600, "SemiBold"), (700, "Bold"))


def download(revision, path):
    url = f"https://raw.githubusercontent.com/google/fonts/{revision}/{quote(path)}"
    with urlopen(url, timeout=60) as response:
        return response.read()


def rename(font, family, weight):
    """Avoid reserved upstream font names in all primary derivative names."""
    names = font["name"]
    ids = (1, 2, 3, 4, 6, 16, 17, 21, 22)
    names.names = [record for record in names.names if record.nameID not in ids]
    family_name = f"Dutchy {family}"
    full_name = f"{family_name} {weight}"
    replacements = {1: family_name, 2: weight, 3: full_name, 4: full_name,
                    6: f"Dutchy{family}-{weight}", 16: family_name, 17: weight}
    for name_id, value in replacements.items():
        names.setName(value, name_id, 3, 1, 0x409)
        names.setName(value, name_id, 1, 0, 0)


def main():
    ROOT.mkdir(parents=True, exist_ok=True)
    manifest_path = ROOT / "manifest.json"
    if manifest_path.exists():
        revision = json.loads(manifest_path.read_text())["google_fonts_revision"]
    else:
        with urlopen("https://api.github.com/repos/google/fonts/commits/main") as response:
            revision = json.load(response)["sha"]
    manifest = {"google_fonts_revision": revision, "fonttools": version, "sources": []}
    for upstream, filename, family in FAMILIES:
        path = f"ofl/{upstream}/{filename}"
        source = download(revision, path)
        (ROOT / f"{family}-OFL.txt").write_bytes(download(revision, f"ofl/{upstream}/OFL.txt"))
        entry = {"path": path, "sha256": hashlib.sha256(source).hexdigest(), "instances": {}}
        for number, weight in WEIGHTS:
            font = TTFont(io.BytesIO(source), recalcTimestamp=False)
            axes = {axis.axisTag: axis.defaultValue for axis in font["fvar"].axes}
            axes["wght"] = number
            font = instantiateVariableFont(font, axes, inplace=True)
            rename(font, family, weight)
            target = ROOT / f"{family}-{weight}.ttf"
            font.save(target)
            entry["instances"][target.name] = hashlib.sha256(target.read_bytes()).hexdigest()
            print(target.name, target.stat().st_size)
        manifest["sources"].append(entry)
    manifest_path.write_text(json.dumps(manifest, indent=2) + "\n")


if __name__ == "__main__":
    main()
