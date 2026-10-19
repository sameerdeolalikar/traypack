//! Deterministic shelf packing for rectangular tabletop components.
//! Coordinates are millimetres. This is a heuristic, not an optimal bin solver.
use std::collections::HashSet;

const EPS: f64 = 1e-8;

#[derive(Clone, Debug)]
pub struct Item {
    pub name: String,
    pub width: f64,
    pub height: f64,
    pub rotate: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct Config {
    pub width: f64,
    pub height: f64,
    pub gap: f64,
    pub margin: f64,
}

impl Config {
    pub fn validate(self) -> Result<(), String> {
        dimension(self.width)?;
        dimension(self.height)?;
        if !self.gap.is_finite() || self.gap < 0.0 || self.gap > 100.0 {
            return Err("gap must be finite and between 0 and 100 mm".into());
        }
        if !self.margin.is_finite()
            || self.margin < 0.0
            || self.margin * 2.0 >= self.width.min(self.height)
        {
            return Err("margin must be nonnegative and leave a usable interior".into());
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct Placement {
    pub name: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub rotated: bool,
}

#[derive(Clone, Debug)]
struct Shelf {
    y: f64,
    height: f64,
    next_x: f64,
}

#[derive(Clone, Debug, Default)]
pub struct Tray {
    pub placements: Vec<Placement>,
    shelves: Vec<Shelf>,
}

fn dimension(value: f64) -> Result<f64, String> {
    if !value.is_finite() || !(0.1..=10000.0).contains(&value) {
        return Err("dimensions must be finite and between 0.1 and 10000 mm".into());
    }
    Ok(value)
}

/// Small explicit CSV dialect: unquoted names, exactly five columns.
pub fn parse_csv(input: &str) -> Result<Vec<Item>, String> {
    let mut lines = input.lines().filter(|line| !line.trim().is_empty());
    if lines.next().map(str::trim) != Some("name,width_mm,height_mm,count,rotate") {
        return Err("CSV header must be name,width_mm,height_mm,count,rotate".into());
    }
    let mut result = Vec::new();
    let mut names = HashSet::new();
    for (index, line) in lines.enumerate() {
        let fields: Vec<&str> = line.split(',').map(str::trim).collect();
        let error = || format!("invalid CSV row {}", index + 2);
        if fields.len() != 5
            || fields[0].is_empty()
            || fields[0].len() > 60
            || fields[0].chars().any(|c| c.is_control() || c == '"')
        {
            return Err(error());
        }
        if !names.insert(fields[0].to_string()) {
            return Err(format!("duplicate component name: {}", fields[0]));
        }
        let width = dimension(fields[1].parse().map_err(|_| error())?)?;
        let height = dimension(fields[2].parse().map_err(|_| error())?)?;
        let count: usize = fields[3].parse().map_err(|_| error())?;
        if count == 0 || count > 1000 || result.len() + count > 1000 {
            return Err("use 1–1000 instances in total".into());
        }
        let rotate = match fields[4] {
            "yes" => true,
            "no" => false,
            _ => return Err("rotate must be yes or no".into()),
        };
        for number in 1..=count {
            result.push(Item {
                name: if count == 1 {
                    fields[0].to_string()
                } else {
                    format!("{} #{number}", fields[0])
                },
                width,
                height,
                rotate,
            });
        }
    }
    if result.is_empty() {
        return Err("CSV must contain at least one component".into());
    }
    Ok(result)
}

fn orientations(item: &Item) -> Vec<(f64, f64, bool)> {
    let mut options = vec![(item.width, item.height, false)];
    if item.rotate && (item.width - item.height).abs() > EPS {
        options.push((item.height, item.width, true));
    }
    options
}

impl Tray {
    fn insert(&mut self, item: &Item, cfg: Config) -> bool {
        let right = cfg.width - cfg.margin;
        let bottom = cfg.height - cfg.margin;
        let mut best: Option<(usize, f64, f64, bool, f64)> = None;
        for (index, shelf) in self.shelves.iter().enumerate() {
            for (w, h, rotated) in orientations(item) {
                let remaining = right - shelf.next_x - w;
                if h <= shelf.height + EPS
                    && remaining >= -EPS
                    && best.as_ref().map_or(true, |b| remaining < b.4)
                {
                    best = Some((index, w, h, rotated, remaining));
                }
            }
        }
        if let Some((index, w, h, rotated, _)) = best {
            let shelf = &mut self.shelves[index];
            self.placements.push(Placement {
                name: item.name.clone(),
                x: shelf.next_x,
                y: shelf.y,
                width: w,
                height: h,
                rotated,
            });
            shelf.next_x += w + cfg.gap;
            return true;
        }
        let y = self
            .shelves
            .last()
            .map_or(cfg.margin, |s| s.y + s.height + cfg.gap);
        let best = orientations(item)
            .into_iter()
            .filter(|(w, h, _)| cfg.margin + w <= right + EPS && y + h <= bottom + EPS)
            .min_by(|a, b| a.1.total_cmp(&b.1).then_with(|| a.0.total_cmp(&b.0)));
        if let Some((w, h, rotated)) = best {
            self.placements.push(Placement {
                name: item.name.clone(),
                x: cfg.margin,
                y,
                width: w,
                height: h,
                rotated,
            });
            self.shelves.push(Shelf {
                y,
                height: h,
                next_x: cfg.margin + w + cfg.gap,
            });
            return true;
        }
        false
    }
}

pub fn pack(items: &[Item], cfg: Config) -> Result<Vec<Tray>, String> {
    cfg.validate()?;
    if items.is_empty() || items.len() > 1000 {
        return Err("use 1–1000 component instances".into());
    }
    for item in items {
        dimension(item.width)?;
        dimension(item.height)?;
        if !orientations(item).iter().any(|(w, h, _)| {
            *w <= cfg.width - 2.0 * cfg.margin + EPS && *h <= cfg.height - 2.0 * cfg.margin + EPS
        }) {
            return Err(format!(
                "{} cannot fit an empty tray with these margins",
                item.name
            ));
        }
    }
    let mut ordered = items.to_vec();
    ordered.sort_by(|a, b| {
        b.width
            .max(b.height)
            .total_cmp(&a.width.max(a.height))
            .then_with(|| (b.width * b.height).total_cmp(&(a.width * a.height)))
            .then_with(|| a.name.cmp(&b.name))
    });
    let mut trays: Vec<Tray> = Vec::new();
    for item in &ordered {
        if trays.iter_mut().any(|tray| tray.insert(item, cfg)) {
            continue;
        }
        let mut tray = Tray::default();
        if !tray.insert(item, cfg) {
            return Err(format!("{} cannot fit a new shelf", item.name));
        }
        trays.push(tray);
    }
    Ok(trays)
}

fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

pub fn svg(trays: &[Tray], cfg: Config) -> String {
    let width = (cfg.width + 20.0).max(180.0);
    let height = 36.0 + trays.len() as f64 * (cfg.height + 24.0);
    let mut out = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="{width}mm" height="{height}mm" viewBox="0 0 {width} {height}" font-family="sans-serif" role="img" aria-labelledby="title">
<title id="title">Traypack component layout</title>
<rect width="100%" height="100%" fill="white"/>
<text x="10" y="13" font-size="5" font-weight="bold">TRAYPACK / COMPONENT LAYOUT</text>
<text x="10" y="20" font-size="3">Shelf heuristic · millimetres · print at actual size · no optimality guarantee</text>
"#
    );
    for (i, tray) in trays.iter().enumerate() {
        let y = 34.0 + i as f64 * (cfg.height + 24.0);
        out.push_str(&format!(r##"<text x="10" y="{}" font-size="3">Tray {} · {} × {} mm · {} items</text>
<g transform="translate(10 {y})"><rect width="{}" height="{}" fill="#faf7ef" stroke="#243744" stroke-width="0.4"/>
"##, y - 3.0, i + 1, cfg.width, cfg.height, tray.placements.len(), cfg.width, cfg.height));
        for p in &tray.placements {
            let label = escape(&p.name);
            out.push_str(&format!(r##"<rect x="{}" y="{}" width="{}" height="{}" fill="#d3e4d8" stroke="#243744" stroke-width="0.3"><title>{label}: {} × {} mm{}</title></rect>
"##, p.x, p.y, p.width, p.height, p.width, p.height, if p.rotated { " (rotated)" } else { "" }));
            if p.width >= 12.0 && p.height >= 6.0 {
                let font =
                    3.0_f64.min((p.width - 2.0) / (p.name.chars().count().max(1) as f64 * 0.65));
                out.push_str(&format!(
                    r#"<text x="{}" y="{}" font-size="{font}" text-anchor="middle">{label}</text>"#,
                    p.x + p.width / 2.0,
                    p.y + p.height / 2.0 + font / 3.0
                ));
            }
        }
        out.push_str("</g>\n");
    }
    out.push_str(&format!(r#"<line x1="10" y1="{}" x2="60" y2="{}" stroke="black" stroke-width="0.3"/><text x="64" y="{}" font-size="3">50 mm calibration</text></svg>
"#, height - 7.0, height - 7.0, height - 6.0));
    out
}
