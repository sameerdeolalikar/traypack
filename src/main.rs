use std::{
    env,
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
    process,
};
use traypack::{pack, parse_csv, svg, Config};

const HELP: &str = "Traypack — rectangular component packing (all dimensions in mm)
Usage: traypack INPUT.csv --width MM --height MM --output layout.svg
Options: --gap MM (default 3), --margin MM (default 5), --no-rotate, --force
CSV header: name,width_mm,height_mm,count,rotate (yes/no)
Output is one custom-sized SVG containing all trays. No optimality guarantee.";

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() || args.iter().any(|a| a == "--help" || a == "-h") {
        println!("{HELP}");
        return Ok(());
    }
    let input = PathBuf::from(&args[0]);
    let (mut width, mut height, mut output) = (None, None, None);
    let (mut gap, mut margin, mut rotate, mut force) = (3.0, 5.0, true, false);
    let mut i = 1;
    while i < args.len() {
        let flag = &args[i];
        match flag.as_str() {
            "--no-rotate" => rotate = false,
            "--force" => force = true,
            "--width" | "--height" | "--gap" | "--margin" | "--output" | "-o" => {
                i += 1;
                let value = args
                    .get(i)
                    .ok_or_else(|| format!("missing value for {flag}"))?;
                if flag == "--output" || flag == "-o" {
                    output = Some(PathBuf::from(value));
                } else {
                    let number: f64 = value
                        .parse()
                        .map_err(|_| format!("invalid number for {flag}"))?;
                    match flag.as_str() {
                        "--width" => width = Some(number),
                        "--height" => height = Some(number),
                        "--gap" => gap = number,
                        _ => margin = number,
                    }
                }
            }
            _ => return Err(format!("unknown option {flag}")),
        }
        i += 1;
    }
    let cfg = Config {
        width: width.ok_or("--width is required")?,
        height: height.ok_or("--height is required")?,
        gap,
        margin,
    };
    let output = output.ok_or("--output is required")?;
    if fs::metadata(&input).map_err(|e| e.to_string())?.len() > 1_000_000 {
        return Err("input exceeds 1 MB".into());
    }
    let mut items = parse_csv(&fs::read_to_string(input).map_err(|e| e.to_string())?)?;
    if !rotate {
        for item in &mut items {
            item.rotate = false;
        }
    }
    let trays = pack(&items, cfg)?;
    let mut options = OpenOptions::new();
    options.write(true);
    if force {
        options.create(true).truncate(true);
    } else {
        options.create_new(true);
    }
    let mut file = options
        .open(&output)
        .map_err(|e| format!("{}: {e}", output.display()))?;
    file.write_all(svg(&trays, cfg).as_bytes())
        .map_err(|e| e.to_string())?;
    let area: f64 = items.iter().map(|i| i.width * i.height).sum();
    println!(
        "{} items → {} trays; {:.1}% of total tray area occupied. Created {}.",
        items.len(),
        trays.len(),
        area / (trays.len() as f64 * cfg.width * cfg.height) * 100.0,
        output.display()
    );
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("traypack: {error}");
        process::exit(2);
    }
}
