use img_parts::jpeg::Jpeg;
use img_parts::png::Png;
use std::fs;
use std::path::PathBuf;
use clap::Parser;

#[derive(Parser)]
struct Cli {
    input: PathBuf,
    #[arg(short, long)]
    output: Option<PathBuf>,
    #[clap(long, short, action)]
    dry_run: bool,
}

fn main() {
    let args = Cli::parse();
    let input = args.input;
    let output = match args.output {
        Some(path) => path,
        None => {
            let stem = input.file_stem().unwrap().to_str().unwrap();
            let ext = input.extension().unwrap().to_str().unwrap();
            PathBuf::from(format!("{stem}_stripped.{ext}"))
        },
    };

    let mut has_exif_xmp = false;
    let mut has_iptc = false;

    let bytes = fs::read(&input).expect("couldnt read file");
    let extension = input
        .extension()
        .unwrap_or_default()
        .to_str()
        .unwrap_or("");

    let out_bytes = match extension {
        "jpg" | "jpeg" => {
            let mut jpeg = Jpeg::from_bytes(bytes.into()).expect("not a valid jpeg");
            jpeg.segments_mut().retain(|seg| {
                let marker = seg.marker();
                match marker {
                    0xED => {
                        has_exif_xmp = true;
                        false
                    },
                    0xE1 => {
                        has_iptc = true;
                        false
                    },
                    _ => true,
                }
            });
            
            jpeg.encoder().bytes()
        }
        "png" => {
            let mut png = Png::from_bytes(bytes.into()).expect("not a valid png");
            png.remove_chunks_by_type([b'e', b'X', b'I', b'f']);
            png.remove_chunks_by_type([b't', b'E', b'X', b't']);
            png.remove_chunks_by_type([b'i', b'T', b'X', b't']);
            png.remove_chunks_by_type([b'z', b'T', b'X', b't']);
            png.encoder().bytes()
        }
        _ => panic!("unsupported file format"),
    };

    if args.dry_run {
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
    } else {
        if !has_exif_xmp && !has_iptc {
            println!("No metadata to strip");
        } else {
            fs::write(&output, out_bytes).expect("cant write to this file");
            println!("done -> {:?}", output);
        }
    }
}