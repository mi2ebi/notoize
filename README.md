# notoize

A crate that tells you what Noto font stack you need.

This is ***not*** "not oize". what's oize

## Beware:

- `notoize()` has to reparse the font support JSONs on every use. It takes around 0.4s on the debug builds. I might make it a `const` *vel sim* in the future.
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