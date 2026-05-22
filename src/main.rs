use img_parts::jpeg::Jpeg;
use img_parts::png::Png;
use std::fs;
use std::path::PathBuf;

fn main() {
    let input = PathBuf::from("test.jpg");
    let output = PathBuf::from("test_o.jpg");

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
                    0xED => false,
                    0xE1 => false,
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
    
    fs::write(&output, out_bytes).expect("cant write to this file");

    println!("done -> {:?}", output);
}