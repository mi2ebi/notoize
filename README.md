# notoize

A crate that tells you what Noto font stack you need.

This is ***not*** "not oize". what's oize

## Beware:

- no config options yet

## `notoize()`

Takes a `&str` and returns a `FontStack`.

## `FontStack`

A `Vec<`font names`>`.

- **`files()`** returns a `Vec<Font>`.

## `Font`

- **`filename`, `bytes`:** for writing the font file elsewhere
- **`fontname`:** for CSS
