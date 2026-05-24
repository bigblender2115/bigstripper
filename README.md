# bigstripper

A CLI tool to strip metadata from image files.
Supports JPEG and PNG. Removes EXIF, XMP, and IPTC data.

### Installation
```bash
git clone https://github.com/yourname/metastrip
cd bigstripper
cargo build --release
```

### Usage
```bash
bigstripper <input> [options]

bigstripper image.jpg
bigstripper image.png --dry-run
bigstripper image.jpg --replace
bigstripper ./photos
```

### Flags
```bash
--output       custom output path
--dry-run      preview what would be stripped without writing
--replace      delete the original file after stripping metadata
```

### What gets stripped
JPEG — APP1 (EXIF, XMP), APP13 (IPTC)
PNG — eXIf, tEXt, iTXt, zTXt chunks
