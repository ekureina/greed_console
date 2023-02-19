use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

/*
 * A console and digital character sheet for campaigns under the greed ruleset.
 * Copyright (C) 2023 Claire Moore
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

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
