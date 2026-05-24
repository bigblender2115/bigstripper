use clap::Parser;
use img_parts::jpeg::Jpeg;
use img_parts::png::Png;
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

#[derive(Parser)]
struct Cli {
    input: PathBuf,
    #[arg(short, long)]
    output: Option<PathBuf>,
    #[clap(long, short, action)]
    dry_run: bool,
    #[clap(long, short, action)]
    replace: bool,
}

fn strip_file(input: &PathBuf, output: &PathBuf, dry_run: bool, replace: bool) {
    let mut has_exif_xmp = false;
    let mut has_iptc = false;

    let bytes = fs::read(&input).expect("couldnt read file");
    let extension = input.extension().unwrap_or_default().to_str().unwrap_or("");

    let original_size = bytes.len();

    let out_bytes = match extension {
        "jpg" | "jpeg" => {
            let mut jpeg = Jpeg::from_bytes(bytes.into()).expect("not a valid jpeg");
            jpeg.segments_mut().retain(|seg| {
                let marker = seg.marker();
                match marker {
                    0xE1 => {
                        has_exif_xmp = true;
                        false
                    }
                    0xED => {
                        has_iptc = true;
                        false
                    }
                    _ => true,
                }
            });

            jpeg.encoder().bytes()
        }
        "png" => {
            let mut png = Png::from_bytes(bytes.into()).expect("not a valid png");
            let chunks = png.chunks();
            has_exif_xmp = chunks.iter().any(|c| c.kind() == [b'e', b'X', b'I', b'f'] || c.kind() == [b'i', b'T', b'X', b't']);
            has_iptc = chunks.iter().any(|c| c.kind() == [b't', b'E', b'X', b't'] || c.kind() == [b'z', b'T', b'X', b't']);
            png.remove_chunks_by_type([b'e', b'X', b'I', b'f']);
            png.remove_chunks_by_type([b't', b'E', b'X', b't']);
            png.remove_chunks_by_type([b'i', b'T', b'X', b't']);
            png.remove_chunks_by_type([b'z', b'T', b'X', b't']);
            png.encoder().bytes()
        }
        _ => {
            println!("skipping {:?}", input);
            return;
        }
    };

    let final_size = out_bytes.len();

    if dry_run {
        if has_exif_xmp {
            println!("exif/xmp chunk found");
        } else {
            println!("exif/xmp chunk not found");
        }
        if has_iptc {
            println!("iptc chunk found");
        } else {
            println!("iptc chunk not found")
        }
        if !has_exif_xmp && !has_iptc {
            println!("No metadata to strip");
        }
        if replace {
            println!("cannot replace in a dry run")
        }
    } else {
        if !has_exif_xmp && !has_iptc {
            println!("No metadata to strip");
            if replace {
                println!("no metadata to strip, nothing to replace");
            }
        } else {
            fs::write(&output, out_bytes).expect("cant write to this file");
            println!("done -> {:?}", output);

            let saved = original_size - final_size;
            let saved_kb = saved as f64 / 1024.0;
            let percent = (saved as f64 / original_size as f64) * 100.0;
            println!("saved {:.2}kb ({:.1}%)", saved_kb, percent);

            if replace {
                fs::remove_file(&input).expect("couldn't delete original file");
            }
        }
    }
}

fn main() {
    let args = Cli::parse();
    let input = args.input;
    let output = match &args.output {
        Some(path) => path.clone(),
        None => {
            let stem = input.file_stem().unwrap().to_str().unwrap();
            let ext = input.extension().unwrap().to_str().unwrap();
            PathBuf::from(format!("{stem}_stripped.{ext}"))
        }
    };

    if input.is_dir() && args.output.is_some() {
        println!("warning: --output ignored for batch metadata stripping (folder mode)");
    }

    if input.is_dir() {
        for thing in WalkDir::new(&input) {
            let thing = thing.unwrap();
            let path = thing.path().to_path_buf();
            let output = match &args.output {
                Some(path) => path.clone(),
                None => {
                    let stem = match path.file_stem().and_then(|s| s.to_str()) {
                        Some(s) => s,
                        None => return,
                    };
                    let ext = match path.extension().and_then(|e| e.to_str()) {
                        Some(e) => e,
                        None => return,
                    };
                    PathBuf::from(format!("{stem}_stripped.{ext}"))
                }
            };
            if path.is_file() {
                strip_file(&path, &output, args.dry_run, args.replace);
            }
        }
    } else {
        strip_file(&input, &output, args.dry_run, args.replace);
    }
}
