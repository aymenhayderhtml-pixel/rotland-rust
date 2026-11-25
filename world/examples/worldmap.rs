//! Renders a top-down biome + hillshade PNG map of the world centred on the camp.
//!
//! Usage:
//! `cargo run --release -p world --example worldmap -- --seed=42 --size=12000 --px=1200 --out=/tmp/worldmap.png`

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::time::Instant;

use glam::{Vec2, Vec3};
use world::biome::registry::BiomeRegistry;
use world::{BiomeId, WorldGenerator};

const DEFAULT_SEED: i32 = 42; // new: DECISIONS 2026-11-25
const DEFAULT_SIZE_M: f32 = 12_000.0; // new: DECISIONS 2026-11-25
const DEFAULT_PX: usize = 1200; // new: DECISIONS 2026-11-25
const DEFAULT_OUT: &str = "/tmp/worldmap.png";

const CAMP_DOT_RADIUS_PX: i32 = 3; // new: DECISIONS 2026-11-25
const CAMP_OUTLINE_RADIUS_PX: i32 = 4; // new: DECISIONS 2026-11-25

struct Args {
    seed: i32,
    size_m: f32,
    px: usize,
    out: PathBuf,
}

fn parse_args() -> Result<Args, String> {
    let mut args = Args {
        seed: DEFAULT_SEED,
        size_m: DEFAULT_SIZE_M,
        px: DEFAULT_PX,
        out: PathBuf::from(DEFAULT_OUT),
    };
    let mut raw = std::env::args().skip(1);
    while let Some(arg) = raw.next() {
        if let Some(v) = arg.strip_prefix("--seed=") {
            args.seed = v.parse().map_err(|_| format!("invalid --seed: {v}"))?;
        } else if arg == "--seed" {
            let v = raw.next().ok_or("--seed requires a value")?;
            args.seed = v.parse().map_err(|_| format!("invalid --seed: {v}"))?;
        } else if let Some(v) = arg.strip_prefix("--size=") {
            args.size_m = v.parse().map_err(|_| format!("invalid --size: {v}"))?;
        } else if arg == "--size" {
            let v = raw.next().ok_or("--size requires a value")?;
            args.size_m = v.parse().map_err(|_| format!("invalid --size: {v}"))?;
        } else if let Some(v) = arg.strip_prefix("--px=") {
            args.px = v.parse().map_err(|_| format!("invalid --px: {v}"))?;
        } else if arg == "--px" {
            let v = raw.next().ok_or("--px requires a value")?;
            args.px = v.parse().map_err(|_| format!("invalid --px: {v}"))?;
        } else if let Some(v) = arg.strip_prefix("--out=") {
            args.out = PathBuf::from(v);
        } else if arg == "--out" {
            let v = raw.next().ok_or("--out requires a value")?;
            args.out = PathBuf::from(v);
        } else if arg == "-h" || arg == "--help" {
            println!(
                "Usage: worldmap [--seed=42] [--size=12000] [--px=1200] [--out=/tmp/worldmap.png]"
            );
            std::process::exit(0);
        } else {
            return Err(format!("unknown argument: {arg}"));
        }
    }
    if args.px == 0 || !args.size_m.is_finite() || args.size_m <= 0.0 {
        return Err("--px and --size must be positive".to_string());
    }
    Ok(args)
}

fn main() {
    let args = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("worldmap: {e}");
            std::process::exit(2);
        }
    };

    let started = Instant::now();
    let generator = match WorldGenerator::from_data_dir(args.seed, paths::repo_path("data")) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("worldmap: failed to build world generator: {e}");
            std::process::exit(1);
        }
    };
    let registry = match BiomeRegistry::load() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("worldmap: failed to load data/biomes.json: {e}");
            std::process::exit(1);
        }
    };

    let camp = generator
        .settlements()
        .first()
        .map_or(Vec2::ZERO, |s| Vec2::new(s.center.x, s.center.z));

    let px = args.px;
    let total = px * px;
    let step = args.size_m / px as f32;
    let half = args.size_m * 0.5;
    let origin_x = camp.x - half;
    let origin_z = camp.y - half;

    let mut heights = vec![0.0f32; total];
    let mut base_rgb = vec![[0u8; 3]; total];
    let mut tiers = vec![0u8; total];
    let mut in_outer = vec![false; total];
    let mut counts = [0usize; BiomeId::ALL.len()];

    for py in 0..px {
        let wz = origin_z + (py as f32 + 0.5) * step;
        let row = py * px;
        for px_i in 0..px {
            let wx = origin_x + (px_i as f32 + 0.5) * step;
            let idx = row + px_i;

            heights[idx] = generator.height(wx, wz);
            let sample = generator.biome_sample(wx, wz);
            counts[sample.primary as usize] += 1;
            tiers[idx] = sample.tier;
            in_outer[idx] = sample.outer > 0.0;

            let c0 = registry.get(sample.primary).map_color;
            let c1 = registry.get(sample.secondary).map_color;
            let t = sample.blend.clamp(0.0, 1.0);
            base_rgb[idx] = [
                mix_u8(c0[0], c1[0], t),
                mix_u8(c0[1], c1[1], t),
                mix_u8(c0[2], c1[2], t),
            ];
        }
    }

    // Hillshade from neighbouring heights so ridges and mountains stand out.
    let sun = Vec3::new(-0.55, 0.72, -0.42).normalize(); // new: DECISIONS 2026-11-25
    let mut rgb = vec![0u8; total * 3];
    for py in 0..px {
        let y_up = py.saturating_sub(1);
        let y_dn = (py + 1).min(px - 1);
        for px_i in 0..px {
            let x_lt = px_i.saturating_sub(1);
            let x_rt = (px_i + 1).min(px - 1);
            let idx = py * px + px_i;

            let h = heights[idx];
            let dx = (heights[py * px + x_rt] - heights[py * px + x_lt]) / (2.0 * step);
            let dz = (heights[y_dn * px + px_i] - heights[y_up * px + px_i]) / (2.0 * step);
            let normal = Vec3::new(-dx, 1.0, -dz).normalize();

            // Combine directional hillshade with gentle elevation lift.
            let hill = 0.55 + 0.55 * normal.dot(sun).clamp(0.0, 1.0); // new: DECISIONS 2026-11-25
            let elev = 0.92 + 0.18 * ((h + 10.0) / 80.0).clamp(0.0, 1.0); // new: DECISIONS 2026-11-25
            let shade = (hill * elev).clamp(0.45, 1.2); // new: DECISIONS 2026-11-25

            let base = base_rgb[idx];
            let out = idx * 3;
            rgb[out] = (base[0] as f32 * shade).round().clamp(0.0, 255.0) as u8;
            rgb[out + 1] = (base[1] as f32 * shade).round().clamp(0.0, 255.0) as u8;
            rgb[out + 2] = (base[2] as f32 * shade).round().clamp(0.0, 255.0) as u8;
        }
    }

    // Draw 1 px ring at core_radius (core -> outer transition) and thin rings at tier borders.
    for py in 0..px.saturating_sub(1) {
        for px_i in 0..px.saturating_sub(1) {
            let idx = py * px + px_i;
            let right = idx + 1;
            let down = idx + px;

            let is_core_edge = in_outer[idx] != in_outer[right] || in_outer[idx] != in_outer[down];
            let is_tier_edge = tiers[idx] != tiers[right] || tiers[idx] != tiers[down];

            if is_core_edge {
                blend_pixel(&mut rgb, idx, [255, 255, 255], 0.85); // new: DECISIONS 2026-11-25
            } else if is_tier_edge {
                blend_pixel(&mut rgb, idx, [30, 24, 20], 0.55); // new: DECISIONS 2026-11-25
            }
        }
    }

    // Draw the camp as a white dot at the centre with a thin dark outline.
    let cx = (px / 2) as i32;
    let cy = (px / 2) as i32;
    for dy in -CAMP_OUTLINE_RADIUS_PX..=CAMP_OUTLINE_RADIUS_PX {
        for dx in -CAMP_OUTLINE_RADIUS_PX..=CAMP_OUTLINE_RADIUS_PX {
            let x = cx + dx;
            let y = cy + dy;
            if x < 0 || y < 0 || x >= px as i32 || y >= px as i32 {
                continue;
            }
            let d2 = dx * dx + dy * dy;
            let idx = y as usize * px + x as usize;
            if d2 <= CAMP_DOT_RADIUS_PX * CAMP_DOT_RADIUS_PX {
                set_pixel(&mut rgb, idx, [255, 255, 255]);
            } else if d2 <= CAMP_OUTLINE_RADIUS_PX * CAMP_OUTLINE_RADIUS_PX {
                set_pixel(&mut rgb, idx, [20, 20, 20]);
            }
        }
    }

    if let Some(parent) = args.out.parent()
        && !parent.as_os_str().is_empty()
        && let Err(e) = std::fs::create_dir_all(parent)
    {
        eprintln!("worldmap: cannot create {}: {e}", parent.display());
        std::process::exit(1);
    }

    if let Err(e) = write_rgb_png(&args.out, px as u32, px as u32, &rgb) {
        eprintln!("worldmap: failed to write {}: {e}", args.out.display());
        std::process::exit(1);
    }

    println!(
        "World map written to {} (seed={}, size={}m, px={}, {:.2}s)",
        args.out.display(),
        args.seed,
        args.size_m,
        px,
        started.elapsed().as_secs_f64()
    );
    println!("Biome area share:");
    for id in BiomeId::ALL {
        let pct = counts[id as usize] as f64 * 100.0 / total as f64;
        println!("  {:<10} {:6.2}%", id.key(), pct);
    }
}

fn mix_u8(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 * (1.0 - t) + b as f32 * t)
        .round()
        .clamp(0.0, 255.0) as u8
}

fn set_pixel(rgb: &mut [u8], idx: usize, color: [u8; 3]) {
    let off = idx * 3;
    rgb[off..off + 3].copy_from_slice(&color);
}

fn blend_pixel(rgb: &mut [u8], idx: usize, overlay: [u8; 3], alpha: f32) {
    let off = idx * 3;
    rgb[off] = mix_u8(rgb[off], overlay[0], alpha);
    rgb[off + 1] = mix_u8(rgb[off + 1], overlay[1], alpha);
    rgb[off + 2] = mix_u8(rgb[off + 2], overlay[2], alpha);
}

/// Self-contained uncompressed RGB8 PNG encoder (stored DEFLATE blocks + IEEE CRC-32 + Adler-32).
fn write_rgb_png(path: &PathBuf, width: u32, height: u32, rgb: &[u8]) -> std::io::Result<()> {
    let row_bytes = width as usize * 3;
    let mut raw = Vec::with_capacity((row_bytes + 1) * height as usize);
    for y in 0..height as usize {
        raw.push(0); // filter type 0 (None)
        let start = y * row_bytes;
        raw.extend_from_slice(&rgb[start..start + row_bytes]);
    }

    let zlib = encode_zlib_stored(&raw);

    let mut ihdr = Vec::with_capacity(13);
    ihdr.extend_from_slice(&width.to_be_bytes());
    ihdr.extend_from_slice(&height.to_be_bytes());
    ihdr.push(8); // bit depth
    ihdr.push(2); // color type: Truecolor (RGB)
    ihdr.push(0); // compression
    ihdr.push(0); // filter
    ihdr.push(0); // interlace

    let file = File::create(path)?;
    let mut w = BufWriter::new(file);
    w.write_all(&[137, 80, 78, 71, 13, 10, 26, 10])?;
    write_png_chunk(&mut w, b"IHDR", &ihdr)?;
    write_png_chunk(&mut w, b"IDAT", &zlib)?;
    write_png_chunk(&mut w, b"IEND", &[])?;
    w.flush()
}

fn encode_zlib_stored(data: &[u8]) -> Vec<u8> {
    const MAX_BLOCK: usize = 65_535;
    let blocks = data.len().div_ceil(MAX_BLOCK).max(1);
    let mut out = Vec::with_capacity(2 + data.len() + blocks * 5 + 4);
    out.push(0x78); // CMF: deflate, 32 KiB window
    out.push(0x01); // FLG: no dict, check bits so (0x7801 % 31 == 0)

    let mut offset = 0;
    while offset < data.len() || offset == 0 {
        let end = (offset + MAX_BLOCK).min(data.len());
        let chunk = &data[offset..end];
        let is_final = end == data.len();
        out.push(u8::from(is_final));
        let len = chunk.len() as u16;
        out.extend_from_slice(&len.to_le_bytes());
        out.extend_from_slice(&(!len).to_le_bytes());
        out.extend_from_slice(chunk);
        offset = end;
        if is_final {
            break;
        }
    }

    let adler = adler32(data);
    out.extend_from_slice(&adler.to_be_bytes());
    out
}

fn write_png_chunk(w: &mut impl Write, kind: &[u8; 4], data: &[u8]) -> std::io::Result<()> {
    w.write_all(&(data.len() as u32).to_be_bytes())?;
    w.write_all(kind)?;
    w.write_all(data)?;
    let crc = crc32(kind, data);
    w.write_all(&crc.to_be_bytes())
}

fn adler32(data: &[u8]) -> u32 {
    const MOD: u32 = 65_521;
    let mut s1: u32 = 1;
    let mut s2: u32 = 0;
    for &b in data {
        s1 = (s1 + u32::from(b)) % MOD;
        s2 = (s2 + s1) % MOD;
    }
    (s2 << 16) | s1
}

fn crc32(kind: &[u8; 4], data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFF_FFFF;
    for &b in kind.iter().chain(data.iter()) {
        crc ^= u32::from(b);
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xEDB8_8320 & mask);
        }
    }
    !crc
}
