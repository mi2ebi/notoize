# notoize

A crate that tells you what Noto font stack you need.

This is ***not*** "not oize". what's oize

## Beware:

- There is a long (depends on your internet speed) delay cloning the font(-data) repos... it's either that or 6gb crate though
- no config options yet
- Sometimes outputs FangsongKSSRotated instead of CJK.

## `notoize()`

Takes a `&str` and returns a `FontStack`.

## `FontStack`

A `Vec<`font names`>`.

- **`files()`** returns a `Vec<Font>`.

## `Font`

- **`filename`, `bytes`:** for writing the font file elsewhere
- **`fontname`:** for CSS
