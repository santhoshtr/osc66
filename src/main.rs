use harfbuzz_rs::{Face, Font, UnicodeBuffer, shape};
use std::io::{self, BufRead, Write};

const ESC: char = '\x1b';
const BEL: char = '\x07';

fn reference_advance(font: &Font) -> i32 {
    let buffer = UnicodeBuffer::new().add_str("0");
    let result = shape(font, buffer, &[]);
    result.get_glyph_positions()[0].x_advance
}

fn process_line(line: &str, font: &Font, ref_advance: i32, out: &mut impl Write) {
    if line.is_empty() {
        writeln!(out).unwrap();
        return;
    }

    let buffer = UnicodeBuffer::new().add_str(line);
    let result = shape(font, buffer, &[]);
    let positions = result.get_glyph_positions();
    let infos = result.get_glyph_infos();
    let input_len = line.len();

    if infos.is_empty() {
        writeln!(out).unwrap();
        return;
    }

    let mut cluster_advances: Vec<(u32, i32)> = Vec::new();
    let mut cur_cluster = infos[0].cluster;
    let mut cur_advance = 0i32;

    for (pos, info) in positions.iter().zip(infos) {
        if info.cluster != cur_cluster {
            cluster_advances.push((cur_cluster, cur_advance));
            cur_cluster = info.cluster;
            cur_advance = 0;
        }
        cur_advance += pos.x_advance;
    }
    cluster_advances.push((cur_cluster, cur_advance));

    let line_bytes = line.as_bytes();

    for (i, &(cluster_start, advance)) in cluster_advances.iter().enumerate() {
        let start = cluster_start as usize;
        let end = if i + 1 < cluster_advances.len() {
            cluster_advances[i + 1].0 as usize
        } else {
            input_len
        };

        if start >= input_len {
            continue;
        }

        let text = std::str::from_utf8(&line_bytes[start..end.min(input_len)]).unwrap_or("");
        if text.is_empty() {
            continue;
        }

        let cells = ((advance as f64) / (ref_advance as f64)).ceil() as i32;
        let cells = cells.clamp(0, 7);

        if cells == 0 {
            continue;
        }

        write!(out, "{}]66;w={};{}{}", ESC, cells, text, BEL).unwrap();
    }

    writeln!(out).unwrap();
}

fn main() {
    let font_name = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "monospace".to_string());

    let mut config = fontconfig::FontConfig::default();
    let font_info = config.find(font_name.clone(), None).unwrap_or_else(|| {
        eprintln!("osc66: font '{}' not found", font_name);
        std::process::exit(1);
    });
    let face = Face::from_file(&font_info.path, 0).expect("Error reading font file.");
    let font = Font::new(face);

    let ref_advance = reference_advance(&font);

    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = stdout.lock();

    for line in stdin.lock().lines() {
        match line {
            Ok(line) => process_line(&line, &font, ref_advance, &mut out),
            Err(e) => {
                eprintln!("osc66: {}", e);
                std::process::exit(1);
            }
        }
    }
}
