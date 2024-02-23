# notoize

A crate that tells you what Noto font stack you need.

This is ***not*** "not oize". what's oize

## Beware:

- no config options yet [wip]
- Sometimes outputs FangsongKSSRotated instead of CJK.
- Fetches CJK when encountering emoji

## Options

- **`font_ext`:** TTF, OTF

The rest: you give a `Vec` of these to the function with the same name. Italics here indicate defaults; where there are several the first is used by `new_sans()` and the second by `prefer_serif()`.

`bool`s:
- **`join_adlam`** *false*

`Serifness` variants (`Sans`, `Serif`):
- **`armenian`**

Other `enum` variants:
- **`lgc`** *Sans*; *Serif*; Mono
- **`arabic`** *Sans*, Kufi; *Naskh*, Nastaliq
-

Everything else has no options.