# packaging/macos/dmg/generate.py

- render · function · L38-L46 — def render(svg, width, out): # The SVG's text is already outlined and no fonts are loaded, so every machine draws the # same pixels.
- quantize · function · L49-L51 — def quantize(im): # No dithering: dither noise compresses badly and the artwork has flat fills.
- srgb_profile · function · L54-L61 — def srgb_profile(): # LittleCMS's built-in sRGB ("No copyright, use freely"), with its creation date fixed.
- write_tiff · function · L64-L98 — def write_tiff(svg, out)
- background_alias · function · L101-L113 — def background_alias(): # What Finder stores for /.background/background.tiff on the volume, minus anything about the # machine (dmgbuild writes the same kind of alias from a mounted image). Disk type 5 (ejectable), # the volume flags and the folder/file ids (25, 26) are what Finder wrote on a test image; the ids # are only hints, as the volume's creation date.
- write_ds_store · function · L116-L147 — def write_ds_store(out)
- main · function · L150-L154 — def main()
