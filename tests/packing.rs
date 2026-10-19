use std::{
    fs,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};
use traypack::{pack, parse_csv, svg, Config, Item};

fn config() -> Config {
    Config {
        width: 160.0,
        height: 110.0,
        gap: 3.0,
        margin: 5.0,
    }
}
fn item(name: &str, w: f64, h: f64, rotate: bool) -> Item {
    Item {
        name: name.into(),
        width: w,
        height: h,
        rotate,
    }
}

#[test]
fn expands_quantities_and_honors_rotation_flags() {
    let items = parse_csv("name,width_mm,height_mm,count,rotate\nDice,18,18,6,no\n").unwrap();
    assert_eq!(items.len(), 6);
    assert_eq!(items[5].name, "Dice #6");
    assert!(!items[0].rotate);
}

#[test]
fn rejects_bad_csv() {
    for row in [
        "A,NaN,20,1,yes",
        "A,10,20,0,yes",
        "A,10,20,1001,no",
        "A,10,20,1,maybe",
        "A,-1,20,1,no",
        "A,10,20,1",
        "\"A,B\",10,20,1,no",
    ] {
        assert!(parse_csv(&format!("name,width_mm,height_mm,count,rotate\n{row}")).is_err());
    }
    assert!(parse_csv("name,width_mm,height_mm,count,rotate\nA,1,1,1,no\nA,2,2,1,no").is_err());
}

#[test]
fn rotation_can_make_an_item_fit() {
    let cfg = Config {
        width: 30.0,
        height: 60.0,
        gap: 0.0,
        margin: 0.0,
    };
    assert!(pack(&[item("deck", 50.0, 20.0, false)], cfg).is_err());
    let trays = pack(&[item("deck", 50.0, 20.0, true)], cfg).unwrap();
    assert!(trays[0].placements[0].rotated);
    assert_eq!(trays[0].placements[0].width, 20.0);
}

#[test]
fn exact_boundary_needs_no_trailing_gap() {
    let cfg = Config {
        width: 100.0,
        height: 50.0,
        gap: 10.0,
        margin: 0.0,
    };
    let trays = pack(
        &[
            item("one", 45.0, 50.0, false),
            item("two", 45.0, 50.0, false),
        ],
        cfg,
    )
    .unwrap();
    assert_eq!(trays.len(), 1);
}

#[test]
fn oversized_component_is_an_error_not_silently_dropped() {
    assert!(pack(&[item("oversize", 200.0, 200.0, true)], config()).is_err());
}

#[test]
fn rejects_invalid_configuration() {
    let mut cfg = config();
    cfg.margin = 100.0;
    assert!(cfg.validate().is_err());
    cfg = config();
    cfg.gap = f64::INFINITY;
    assert!(cfg.validate().is_err());
    cfg = config();
    cfg.width = f64::NAN;
    assert!(cfg.validate().is_err());
}

#[test]
fn generated_layouts_preserve_every_item_and_clearance() {
    let mut seed = 42_u64;
    for _ in 0..40 {
        let mut items = Vec::new();
        for n in 0..35 {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let w = 5.0 + (seed % 500) as f64 / 10.0;
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let h = 5.0 + (seed % 500) as f64 / 10.0;
            items.push(item(&format!("part{n}"), w, h, true));
        }
        let cfg = config();
        let trays = pack(&items, cfg).unwrap();
        assert_eq!(
            trays.iter().map(|t| t.placements.len()).sum::<usize>(),
            items.len()
        );
        let mut names = std::collections::HashSet::new();
        for tray in trays {
            for (index, a) in tray.placements.iter().enumerate() {
                assert!(names.insert(a.name.clone()));
                assert!(a.x >= cfg.margin && a.y >= cfg.margin);
                assert!(a.x + a.width <= cfg.width - cfg.margin + 1e-8);
                assert!(a.y + a.height <= cfg.height - cfg.margin + 1e-8);
                for b in &tray.placements[index + 1..] {
                    assert!(
                        a.x + a.width + cfg.gap <= b.x + 1e-8
                            || b.x + b.width + cfg.gap <= a.x + 1e-8
                            || a.y + a.height + cfg.gap <= b.y + 1e-8
                            || b.y + b.height + cfg.gap <= a.y + 1e-8
                    );
                }
            }
        }
    }
}

#[test]
fn deterministic_across_input_order() {
    let mut items = vec![item("a", 30.0, 40.0, true), item("b", 12.0, 45.0, true)];
    let first = svg(&pack(&items, config()).unwrap(), config());
    items.reverse();
    assert_eq!(first, svg(&pack(&items, config()).unwrap(), config()));
}

#[test]
fn svg_escapes_names_and_uses_physical_units() {
    let cfg = config();
    let output = svg(
        &pack(&[item("<deck & dice>", 40.0, 60.0, true)], cfg).unwrap(),
        cfg,
    );
    assert!(output.contains("&lt;deck &amp; dice&gt;"));
    assert!(output.contains("width=\"180mm\""));
    assert!(!output.contains("<deck"));
}

#[test]
fn cli_protects_existing_output_and_returns_nonzero_on_invalid_input() {
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    let dir = std::env::temp_dir().join(format!(
        "traypack-test-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&dir).unwrap();
    let input = dir.join("input.csv");
    let output = dir.join("output.svg");
    fs::write(
        &input,
        "name,width_mm,height_mm,count,rotate\nDeck,60,40,1,yes\n",
    )
    .unwrap();
    let call = |force: bool| {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_traypack"));
        cmd.arg(&input)
            .args(["--width", "160", "--height", "110", "--output"])
            .arg(&output);
        if force {
            cmd.arg("--force");
        }
        cmd.output().unwrap()
    };
    assert!(call(false).status.success());
    fs::write(&output, "keep me").unwrap();
    assert_eq!(call(false).status.code(), Some(2));
    assert_eq!(fs::read_to_string(&output).unwrap(), "keep me");
    assert!(call(true).status.success());
    fs::write(&input, "bad csv").unwrap();
    assert_eq!(call(true).status.code(), Some(2));
    fs::remove_dir_all(dir).unwrap();
}
