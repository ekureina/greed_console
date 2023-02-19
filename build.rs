use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=greed_console_icon.svg");
    println!("cargo:rerun-if-changed=build.rs");
    let out_dir = env::var("OUT_DIR").unwrap();
    let svg_path = Path::new("greed_console_icon.svg");
    let image = nsvg::parse_file(svg_path, nsvg::Units::Pixel, 96.0).unwrap();
    let (width, height, data) = image.rasterize_to_raw_rgba(1.0).unwrap();
    let mut file = File::create(out_dir + "greed_console_icon").unwrap();
    file.write_all(&data).unwrap();
    println!("cargo:rustc-env=GREED_CONSOLE_ICON_WIDTH={}", width);
    println!("cargo:rustc-env=GREED_CONSOLE_ICON_HEIGHT={}", height);
}
