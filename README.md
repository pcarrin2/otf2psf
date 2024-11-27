# otf2psf

This tool creates a PSF2 Linux console font from a modern OTF/TTF monospace font.

It works on fonts that contain embedded bitmap strikes, as well as those with only vector characters. 

## Why?

I wanted a TTY font with some rarely-used box-drawing characters. I don't speak/write languages other than English, so I didn't need any of the usual Latin characters with diacritics. I also specifically wanted a TTY version of GNU Unifont, and the only one they package is for APL. Hence, this project was born.

## Installation

Clone this repo, `cargo build --release`, and copy the output binary to /bin if desired.

## Usage

You will need three components to generate a PSF2 font from an OTF/TTF font using this tool:
- A compiled binary of this tool
- An OTF/TTF font (two examples are included, a Terminus TTF and a GNU Unifont OTF)
- A list of characters to include in the finished font (an example is included, `example.set`).

With these ingredients:

`otf2psf convert in.otf out.psf 12 -u example.set`

rasterizes in.otf at 12pt, using example.set as the Unicode character set, writing the psf2 font to out.psf.

If you get an error that not all characters in the font have the same size, you can use the `--pad` flag to pad out all glyphs to the size of the largest glyph.

If `--pad` makes most glyphs look way too far apart, try the `report` subcommand to view the size of each glyph. Then you can remove the ones that are too big from the charset:

`otf2psf report in.otf 12 --unicode-table-file example.set`

If the glyphs just look weird, missing parts, lumpy, etc -- you're probably trying to rasterize the font at a size where it can't be rendered pixel-perfectly. Try adjusting the size, and if the situation doesn't improve, choose a different font.

## Charset file format

See `example.set` for a valid example charset. Comments beginning with `#` and blank lines are ignored. Each line corresponds to a glyph in the finished PSF2 font, and can contain one or more Unicode codepoints, written as "U+[hex]". 

One glyph can map to multiple Unicode characters or even character sequences. For example this line:

```
U+00E9, U+0065 U+0301
```

means that the same glyph should be used to represent the single character `U+00E9` (LATIN SMALL LETTER E WITH ACUTE) and the sequence `U+0065 U+0301` (ASCII lowercase e + combining acute accent). 

## Caveats

Technically you can create a PSF2 font with whatever characters you'd like. However, if you want your font to be usable by the Linux kernel, there are some constraints you should obey.

- Due to a nofix kernel bug, blank space in the TTY is filled with the 32nd (0x20th) glyph in the PSF2 font, regardless of whether that glyph is actually mapped to space or not. So the 32nd listed character in your charset should be U+0020 (space).
- There are kernel-imposed size constraints on the glyphset portion of the font. The kernel can support a maximum of 512 glyphs and a max glyph size of 64x128 pixels. (I think. I'm skimming the kbd source code without fully understanding it.)
- Some very low-level functions, eg SysReq functions, will disregard the font's Unicode mapping table when printing ASCII. For best compatibility, preserve the indices of all ASCII characters in the Unicode table. (For example, the 41st (0x65th) listed character should be `U+0065`, or capital A). I didn't do that in `example.set` because I'm lazy.
