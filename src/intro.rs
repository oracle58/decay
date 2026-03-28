use std::time::{Duration, Instant};

use crate::term::Term;

const BG: (u8, u8, u8) = (10, 10, 16);
const BRANCH_FG: (u8, u8, u8) = (100, 80, 60);
const TRUNK_FG: (u8, u8, u8) = (75, 60, 45);
const TITLE_FG: (u8, u8, u8) = (140, 135, 125);

const LEAF_PALETTE: [(u8, u8, u8); 8] = [
    (55, 155, 55),
    (95, 165, 40),
    (170, 155, 30),
    (195, 135, 25),
    (205, 100, 25),
    (185, 60, 30),
    (155, 40, 35),
    (115, 70, 45),
];

const LEAF_CHARS: [char; 6] = ['@', '&', '#', '%', '*', 'o'];

// Bare branch crown revealed as leaves blow away
const BRANCHES: &[&str] = &[
    r"  \   \       /   /  ",
    r"   \   \     /   /   ",
    r"    \   \   /   /    ",
    r"     \   \_/   /     ",
    r"      \   |   /      ",
    r"       \  |  /       ",
    r"        \_|_/        ",
];

const TRUNK: &[&str] = &[
    r"          |          ",
    r"          |          ",
    r"          |          ",
    r"         /|\         ",
    r"    ____/ | \____    ",
];

const TITLE: &[&str] = &[
    r" ____   _____   ____     _    __   __",
    r"|  _ \ | ____| / ___|   / \   \ \ / /",
    r"| | | ||  _|  | |      / _ \   \ V / ",
    r"| |_| || |___ | |___  / ___ \   | |  ",
    r"|____/ |_____| \____|/_/   \_\  |_|  ",
];

const STRUCT_W: usize = 21;
const CANOPY_H: usize = 13;

fn hash(a: usize, b: usize) -> u32 {
    let mut x = (a as u32).wrapping_mul(2654435761).wrapping_add(b as u32).wrapping_mul(2246822519);
    x ^= x >> 13;
    x ^= x << 7;
    x ^= x >> 17;
    x
}

fn xorshift(state: &mut u32) -> u32 {
    let mut x = *state;
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    *state = x;
    x
}

fn randf(rng: &mut u32) -> f32 {
    (xorshift(rng) & 0xFFFF) as f32 / 65536.0
}

struct Leaf {
    row: u16,
    col: u16,
    ch: char,
    fg: (u8, u8, u8),
    removal_key: f32,
}

struct Particle {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    ch: char,
    fg: (u8, u8, u8),
    life: u8,
}

pub fn play(term: &mut Term) {
    let cols = term.cols() as usize;
    let rows = term.rows() as usize;
    if cols < 40 || rows < 20 {
        return;
    }

    let title_w = TITLE.iter().map(|l| l.chars().count()).max().unwrap_or(0);
    let total_h = CANOPY_H + TRUNK.len() + 2 + TITLE.len();
    let start_y = rows.saturating_sub(total_h) / 2;
    let struct_x = cols.saturating_sub(STRUCT_W) / 2;
    let center_col = struct_x + STRUCT_W / 2;

    // Branch cells (within canopy area, revealed as leaves fall)
    let branch_y = start_y + CANOPY_H - BRANCHES.len();
    let mut structure: Vec<(u16, u16, char, (u8, u8, u8))> = Vec::new();
    for (i, line) in BRANCHES.iter().enumerate() {
        for (j, ch) in line.chars().enumerate() {
            if ch != ' ' {
                structure.push(((branch_y + i) as u16, (struct_x + j) as u16, ch, BRANCH_FG));
            }
        }
    }

    // Trunk cells (below canopy, always visible)
    let trunk_y = start_y + CANOPY_H;
    for (i, line) in TRUNK.iter().enumerate() {
        for (j, ch) in line.chars().enumerate() {
            if ch != ' ' {
                structure.push(((trunk_y + i) as u16, (struct_x + j) as u16, ch, TRUNK_FG));
            }
        }
    }

    // Title cells
    let title_x = cols.saturating_sub(title_w) / 2;
    let title_y = trunk_y + TRUNK.len() + 2;
    let title_cells: Vec<(u16, u16, char)> = TITLE
        .iter()
        .enumerate()
        .flat_map(|(i, line)| {
            line.chars()
                .enumerate()
                .filter(|&(_, ch)| ch != ' ')
                .map(move |(j, ch)| ((title_y + i) as u16, (title_x + j) as u16, ch))
        })
        .collect();

    // Generate leaves in canopy ellipse
    let canopy_cx = center_col as f32;
    let canopy_cy = start_y as f32 + CANOPY_H as f32 * 0.45;
    let canopy_rx: f32 = 18.0;
    let canopy_ry: f32 = CANOPY_H as f32 * 0.52;

    let mut rng: u32 = 0xDECA_7135;
    let mut leaves: Vec<Leaf> = Vec::new();

    let scan_left = (canopy_cx - canopy_rx - 2.0).max(0.0) as usize;
    let scan_right = ((canopy_cx + canopy_rx + 2.0) as usize).min(cols);

    for r in start_y..(start_y + CANOPY_H) {
        for c in scan_left..scan_right {
            let dy = (r as f32 - canopy_cy) / canopy_ry;
            let dx = (c as f32 - canopy_cx) / canopy_rx;
            let d2 = dx * dx + dy * dy;

            let threshold = 0.80 + randf(&mut rng) * 0.20;
            if d2 > threshold {
                continue;
            }
            if randf(&mut rng) < 0.12 + d2 * 0.18 {
                continue;
            }

            let h = hash(r, c);
            let ch = LEAF_CHARS[(h as usize) % LEAF_CHARS.len()];
            let autumn = (d2 * 6.0).min(5.0) as usize;
            let base = (h as usize / 3) % 3;
            let ci = (base + autumn).min(LEAF_PALETTE.len() - 1);
            let jitter = randf(&mut rng) * 0.35;

            leaves.push(Leaf {
                row: r as u16,
                col: c as u16,
                ch,
                fg: LEAF_PALETTE[ci],
                removal_key: d2 + jitter,
            });
        }
    }

    // Outer leaves removed first
    leaves.sort_by(|a, b| b.removal_key.partial_cmp(&a.removal_key).unwrap());

    if leaves.is_empty() {
        return;
    }

    let total_leaves = leaves.len();
    let frame_dur = Duration::from_millis(16);

    // Step 1: Fade in full tree
    {
        fill_bg(term, cols, rows);
        term.flush().ok();

        let dur = Duration::from_millis(700);
        let start = Instant::now();
        loop {
            let t = (start.elapsed().as_secs_f64() / dur.as_secs_f64()).min(1.0);
            let b = 1.0 - (1.0 - t) * (1.0 - t);

            fill_bg(term, cols, rows);
            draw_structure(term, &structure, b);
            draw_leaves(term, &leaves, 0, total_leaves, b);
            term.reset();
            term.flush().ok();

            if t >= 1.0 {
                break;
            }
            std::thread::sleep(frame_dur);
        }
    }

    // Step 2: Hold full tree
    std::thread::sleep(Duration::from_millis(500));

    // Step 3: Leaves blow away
    {
        let blow_dur = Duration::from_millis(2800);
        let start = Instant::now();
        let mut removed = 0usize;
        let mut particles: Vec<Particle> = Vec::new();

        loop {
            let elapsed = start.elapsed();
            let t = (elapsed.as_secs_f64() / blow_dur.as_secs_f64()).min(1.0);
            let ease = t * t;
            let target = (ease * total_leaves as f64) as usize;

            while removed < target.min(total_leaves) {
                let leaf = &leaves[removed];
                particles.push(Particle {
                    x: leaf.col as f32,
                    y: leaf.row as f32,
                    vx: 1.5 + randf(&mut rng) * 3.5,
                    vy: -0.8 + randf(&mut rng) * 1.8,
                    ch: leaf.ch,
                    fg: leaf.fg,
                    life: 18 + (xorshift(&mut rng) % 18) as u8,
                });
                removed += 1;
            }

            tick_particles(&mut particles, &mut rng, cols, rows);

            fill_bg(term, cols, rows);
            draw_structure(term, &structure, 1.0);
            draw_leaves(term, &leaves, removed, total_leaves, 1.0);
            draw_particles(term, &particles);
            term.reset();
            term.flush().ok();

            if t >= 1.0 && particles.is_empty() {
                break;
            }
            std::thread::sleep(frame_dur);
        }
    }

    // Step 4: Title fade in
    {
        let dur = Duration::from_millis(400);
        let start = Instant::now();
        loop {
            let t = (start.elapsed().as_secs_f64() / dur.as_secs_f64()).min(1.0);

            fill_bg(term, cols, rows);
            draw_structure(term, &structure, 1.0);
            draw_title(term, &title_cells, t);
            term.reset();
            term.flush().ok();

            if t >= 1.0 {
                break;
            }
            std::thread::sleep(frame_dur);
        }
    }

    // Step 5: Hold bare tree + title
    std::thread::sleep(Duration::from_millis(800));

    // Step 6: Fade out
    {
        let steps = 8u8;
        let step_dur = Duration::from_millis(50);
        for s in 1..=steps {
            let frac = 1.0 - s as f64 / steps as f64;
            fill_bg(term, cols, rows);
            draw_structure(term, &structure, frac);
            draw_title(term, &title_cells, frac);
            term.reset();
            term.flush().ok();
            std::thread::sleep(step_dur);
        }
    }

    term.clear();
    term.flush().ok();
}

fn fill_bg(term: &mut Term, cols: usize, rows: usize) {
    term.bg_rgb(BG.0, BG.1, BG.2).fg_rgb(BG.0, BG.1, BG.2);
    for r in 0..rows {
        term.move_to(0, r as u16).print_n(' ', cols);
    }
}

fn draw_structure(term: &mut Term, cells: &[(u16, u16, char, (u8, u8, u8))], brightness: f64) {
    term.bg_rgb(BG.0, BG.1, BG.2);
    for &(r, c, ch, fg) in cells {
        term.move_to(c, r)
            .fg_rgb(
                (fg.0 as f64 * brightness) as u8,
                (fg.1 as f64 * brightness) as u8,
                (fg.2 as f64 * brightness) as u8,
            )
            .print_char(ch);
    }
}

fn draw_leaves(term: &mut Term, leaves: &[Leaf], from: usize, to: usize, brightness: f64) {
    term.bg_rgb(BG.0, BG.1, BG.2);
    for leaf in &leaves[from..to] {
        term.move_to(leaf.col, leaf.row)
            .fg_rgb(
                (leaf.fg.0 as f64 * brightness) as u8,
                (leaf.fg.1 as f64 * brightness) as u8,
                (leaf.fg.2 as f64 * brightness) as u8,
            )
            .print_char(leaf.ch);
    }
}

fn draw_particles(term: &mut Term, particles: &[Particle]) {
    term.bg_rgb(BG.0, BG.1, BG.2);
    for p in particles {
        let fade = (p.life as f32 / 36.0).min(1.0);
        term.move_to(p.x as u16, p.y as u16)
            .fg_rgb(
                (p.fg.0 as f32 * fade) as u8,
                (p.fg.1 as f32 * fade) as u8,
                (p.fg.2 as f32 * fade) as u8,
            )
            .print_char(p.ch);
    }
}

fn draw_title(term: &mut Term, cells: &[(u16, u16, char)], brightness: f64) {
    term.bg_rgb(BG.0, BG.1, BG.2);
    for &(r, c, ch) in cells {
        term.move_to(c, r)
            .fg_rgb(
                (TITLE_FG.0 as f64 * brightness) as u8,
                (TITLE_FG.1 as f64 * brightness) as u8,
                (TITLE_FG.2 as f64 * brightness) as u8,
            )
            .print_char(ch);
    }
}

fn tick_particles(particles: &mut Vec<Particle>, rng: &mut u32, cols: usize, rows: usize) {
    for p in particles.iter_mut() {
        p.x += p.vx * 0.3;
        p.y += p.vy * 0.3;
        p.vy += 0.06;
        if p.life > 0 {
            p.life -= 1;
        }
        if xorshift(rng) % 10 == 0 {
            p.ch = LEAF_CHARS[(xorshift(rng) as usize) % LEAF_CHARS.len()];
        }
    }
    particles.retain(|p| {
        p.life > 0 && p.x >= 0.0 && (p.x as usize) < cols && p.y >= 0.0 && (p.y as usize) < rows
    });
}
