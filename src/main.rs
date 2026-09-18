//! alchemy — a chemistry bench in the terminal.
//!
//! Four glasses on a bench. Pour things in from the shelf, light the
//! burner, and watch: the colour of the liquid, what settles out of it,
//! what bubbles away, how warm it gets and how acid it is. None of it is
//! canned. Every reaction runs at a speed set by how crowded its
//! reactants are, and the picture is drawn from whatever is left.
//!
//! When nothing is happening, nothing runs. The app blocks on a key and
//! the chemistry only ticks while a glass still has something to do.

mod chem;
mod lessons;

use std::path::PathBuf;
use std::time::{Duration, Instant};

use chem::{ph_word, species, Glass, Rgb, Vessel, PER_ML};
use crust::cursor::Cursor;
use crust::{style, Crust, Input, Pane};
use lessons::LESSONS;

/// Width of the panel on the right.
const INFO_W: usize = 44;
/// How often the screen follows a working bench.
const TICK_MS: u64 = 100;
/// Seconds of one tick, for the chemistry.
const DT: f64 = 0.1;
/// How many glasses stand on the bench.
const GLASSES: usize = 4;
/// Width of one glass's place on the bench.
const SLOT: usize = 15;
/// How tall a glass is drawn, in rows.
const GH: usize = 12;
/// Rows of vapour space above the glasses.
const VAP: usize = 2;

/// One thing on the shelf: how much of it a single measure is, and how
/// much water comes with it. A solution brings its water; a powder does
/// not.
struct Item {
    key: &'static str,
    label: &'static str,
    mmol: f64,
    ml: f64,
}

const fn sol(key: &'static str, label: &'static str) -> Item { Item { key, label, mmol: 10.0, ml: 10.0 } }
const fn pow(key: &'static str, label: &'static str) -> Item { Item { key, label, mmol: 5.0, ml: 0.0 } }
const fn drop_(key: &'static str, label: &'static str) -> Item { Item { key, label, mmol: 0.05, ml: 0.0 } }

static SHELF: &[(&str, &[Item])] = &[
    ("Water and such", &[
        Item { key: "H2O", label: "water", mmol: 0.0, ml: 15.0 },
        sol("starch", "starch water"),
        sol("sugar", "sugar"),
        Item { key: "H2O2", label: "hydrogen peroxide", mmol: 40.0, ml: 10.0 },
    ]),
    ("Acids", &[
        sol("HCl", "hydrochloric acid"),
        sol("H2SO4", "sulfuric acid"),
        sol("HNO3", "nitric acid"),
        sol("AcOH", "vinegar"),
    ]),
    ("Bases", &[
        sol("NaOH", "caustic soda"),
        sol("KOH", "caustic potash"),
        sol("NH3", "ammonia"),
        sol("CaOH2", "limewater"),
    ]),
    ("Salts", &[
        sol("NaCl", "table salt"),
        sol("KCl", "potassium chloride"),
        sol("KNO3", "saltpetre"),
        sol("NaNO3", "sodium nitrate"),
        sol("Na2CO3", "washing soda"),
        sol("NaHCO3", "baking soda"),
        sol("Na2SO4", "sodium sulfate"),
        sol("CaCl2", "calcium chloride"),
        pow("CaCO3", "chalk"),
    ]),
    ("Salts with a colour", &[
        sol("CuSO4", "copper sulfate"),
        pow("CuSO4d", "copper sulfate, baked dry"),
        sol("CuCl2", "copper chloride"),
        sol("FeCl3", "iron chloride"),
        sol("KMnO4", "permanganate"),
        sol("K2Cr2O7", "dichromate"),
        pow("CoCl2d", "cobalt chloride, dry"),
        sol("NiSO4", "nickel sulfate"),
    ]),
    ("Salts for the tests", &[
        sol("AgNO3", "silver nitrate"),
        sol("BaCl2", "barium chloride"),
        sol("BaNO3", "barium nitrate"),
        sol("PbNO3", "lead nitrate"),
        sol("KI", "potassium iodide"),
        sol("NaI", "sodium iodide"),
        sol("Na2S2O3", "hypo"),
        sol("SrCl2", "strontium chloride"),
        sol("LiCl", "lithium chloride"),
    ]),
    ("Metals", &[
        pow("Cu", "copper"),
        pow("Zn", "zinc"),
        pow("Fe", "iron"),
        pow("Mg", "magnesium"),
        pow("Al", "aluminium"),
        pow("Na", "sodium"),
    ]),
    ("Powders and fuels", &[
        pow("S", "sulfur"),
        pow("C", "charcoal"),
        pow("I2", "iodine"),
        pow("Pred", "red phosphorus"),
        pow("KClO3", "potassium chlorate"),
        pow("Fe2O3", "iron oxide"),
    ]),
    ("Indicators", &[
        drop_("litmus", "litmus"),
        drop_("phen", "phenolphthalein"),
        drop_("uni", "universal indicator"),
        drop_("cabbage", "red cabbage juice"),
    ]),
    ("For the clock", &[
        sol("KIO3", "potassium iodate"),
        Item { key: "MA", label: "malonic acid", mmol: 50.0, ml: 10.0 },
        Item { key: "MnSO4", label: "manganese sulfate", mmol: 1.0, ml: 5.0 },
    ]),
];

const HELP: &str = "\
 alchemy — a chemistry bench

 ← →  or 1-4   pick a glass
 SPACE         the shelf: pour something in
 w             wash the glass out
 v             swap the glass: tube, beaker, flask
 h             the burner, on and off
 t             a flame test: dip the wire and hold it in the flame
 s             the safety screen, up or down

 x             the experiments
 n             the next experiment
 ?             this
 q             quit

 The panel on the right names everything in the glass you have picked,
 how warm it is and how acid it is. Every reaction you see runs at a
 speed set by how crowded it is, so a thin solution takes its time.

 Nothing runs while nothing is happening. The moment the bench settles,
 alchemy blocks and waits for you.

 press any key";

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "-h" || a == "--help") {
        println!("alchemy — a chemistry bench in the terminal (Fe2O3 suite)");
        println!();
        println!("Usage: alchemy");
        println!();
        println!("Four glasses, a shelf of reagents and a burner. Pour things together and");
        println!("watch the colour, the precipitate, the bubbles, the heat and the pH. Ten");
        println!("experiments run from a blue solution to a clock that swings blue and clear.");
        println!("Progress is kept in ~/.alchemy/. Press ? inside for every key.");
        return;
    }
    if args.iter().any(|a| a == "-v" || a == "--version") {
        println!("alchemy {}", env!("CARGO_PKG_VERSION"));
        return;
    }

    Crust::init();
    Crust::set_app_identity("Alchemy");
    Crust::clear_screen();
    let mut app = App::new();
    app.render();
    loop {
        let key = if app.working() { Input::getchr_ms(TICK_MS) } else { Input::getchr(None) };
        if let Some(k) = key {
            if app.key(&k) { break; }
        }
        app.tick();
        app.render();
    }
    app.save();
    Crust::cleanup();
}

struct App {
    bench: Vec<Vessel>,
    cur: usize,
    /// 0 is free play; 1 and up are the experiments.
    mode: usize,
    done: Vec<bool>,
    note: Option<String>,
    /// The flame test showing right now, and when it burns out.
    flame: Option<(Rgb, &'static str, Instant)>,
    /// The safety screen. Down is how everyone starts, which is the whole
    /// trouble.
    screen: bool,
    /// Set when a flash went off with your face over it.
    blind: Option<Instant>,
    frame: u64,
    shown: [String; 2],
    rows: Vec<String>,
    dirty: bool,
}

impl App {
    fn new() -> App {
        let mode = std::fs::read_to_string(dir().join("mode")).ok()
            .and_then(|s| s.trim().parse().ok()).filter(|&m| m <= LESSONS.len()).unwrap_or(1);
        App {
            bench: (0..GLASSES).map(|i| Vessel::new(if i == 3 { Glass::Beaker } else { Glass::Tube })).collect(),
            cur: 0,
            mode,
            done: lessons::load_done(&dir().join("done")),
            note: None,
            flame: None,
            screen: false,
            blind: None,
            frame: 0,
            shown: Default::default(),
            rows: Vec::new(),
            dirty: true,
        }
    }

    fn save(&self) {
        let _ = std::fs::create_dir_all(dir());
        let _ = std::fs::write(dir().join("mode"), self.mode.to_string());
        lessons::save_done(&dir().join("done"), &self.done);
    }

    /// True while anything on the bench still has something to do.
    fn working(&self) -> bool {
        self.blind.is_some() || self.flame.is_some() || self.bench.iter().any(|v| v.busy())
    }

    fn tick(&mut self) {
        if let Some(when) = self.blind {
            if when.elapsed() > Duration::from_secs(4) {
                self.blind = None;
                self.note = Some("Eighteen minutes, in the real thing, before the sight came back. \
                                  Press s for the screen.".into());
            }
            self.frame += 1;
            self.dirty = true;
            return;
        }
        if let Some((_, _, out)) = self.flame {
            if Instant::now() > out { self.flame = None; }
            self.dirty = true;
        }
        let mut banged = false;
        for v in &mut self.bench {
            if !v.busy() { continue; }
            for e in v.step(DT) {
                if e.bang { banged = true; }
                self.note = Some(e.says);
            }
            self.dirty = true;
        }
        if banged && !self.screen && self.blind.is_none() {
            self.blind = Some(Instant::now());
        }
        self.frame += 1;
        if self.mode > 0 && !self.done[self.mode] {
            let lesson = &LESSONS[self.mode - 1];
            if self.bench.iter().any(|v| (lesson.done)(v)) {
                self.done[self.mode] = true;
                self.note = Some(format!("Experiment {} done: {}. Press n for the next one.",
                                         self.mode, lesson.title));
                self.save();
            }
        }
    }

    // ── Keys ───────────────────────────────────────────────────────────

    /// True to quit.
    fn key(&mut self, k: &str) -> bool {
        self.dirty = true;
        if self.blind.is_some() { return false; }
        match k {
            "q" => return true,
            "LEFT" => self.cur = (self.cur + GLASSES - 1) % GLASSES,
            "RIGHT" => self.cur = (self.cur + 1) % GLASSES,
            "1" | "2" | "3" | "4" => self.cur = k.parse::<usize>().unwrap() - 1,
            " " | "ENTER" => self.shelf(),
            "w" => { self.bench[self.cur].empty(); self.note = Some("washed out".into()); }
            "v" => {
                let g = self.bench[self.cur].glass.next();
                self.bench[self.cur] = Vessel::new(g);
                self.note = Some(format!("a {} now", g.name()));
            }
            "h" => {
                let on = !self.bench[self.cur].burner;
                self.bench[self.cur].burner = on;
                self.note = Some(if on { "the burner is lit".into() } else { "burner off".to_string() });
            }
            "t" => self.flame_test(),
            "s" => {
                self.screen = !self.screen;
                self.note = Some(if self.screen { "the safety screen is up".into() }
                                 else { "the screen is down; nothing between you and the glass".to_string() });
            }
            "x" => self.pick_experiment(),
            "n" => {
                self.mode = if self.mode >= LESSONS.len() { 0 } else { self.mode + 1 };
                self.note = None;
                self.save();
            }
            "?" => self.popup(HELP, 66),
            _ => {}
        }
        false
    }

    fn flame_test(&mut self) {
        match self.bench[self.cur].flame() {
            Some((rgb, what)) => {
                self.flame = Some((rgb, what, Instant::now() + Duration::from_secs(4)));
                self.note = Some(what.to_string());
            }
            None => self.note = Some("nothing in there paints a flame".into()),
        }
    }

    /// The shelf: pick a group, then pick what to pour in.
    fn shelf(&mut self) {
        let groups: Vec<String> = SHELF.iter().map(|(name, items)| format!(" {name}  ({})", items.len())).collect();
        let g = self.menu("The shelf", &groups);
        // Put the bench back before the second menu opens over it.
        self.redraw();
        self.render();
        let Some(g) = g else { return };
        let items = SHELF[g].1;
        let lines: Vec<String> = items.iter().map(|i| {
            let f = species(i.key).map(|s| s.formula).unwrap_or("");
            format!(" {:<26} {}", i.label, f)
        }).collect();
        let n = self.menu(SHELF[g].0, &lines);
        self.redraw();
        let Some(n) = n else { return };
        let item = &items[n];
        let v = &mut self.bench[self.cur];
        if item.ml > 0.0 { v.add("H2O", item.ml * PER_ML); }
        if item.mmol > 0.0 { v.add(item.key, item.mmol); }
        self.note = Some(match species(item.key) {
            Some(sp) => sp.note.to_string(),
            None => format!("{} in", item.label),
        });
    }

    fn pick_experiment(&mut self) {
        let mut lines = vec![" Free play  (no goal, the whole shelf)".to_string()];
        for (i, l) in LESSONS.iter().enumerate() {
            let tick = if self.done[i + 1] { "✓" } else { " " };
            lines.push(format!(" {tick} {}. {}", i + 1, l.title));
        }
        let mut p = self.lower_right(60, lines.len() as u16 + 2);
        p.pane.index = self.mode;
        if let Some(n) = p.modal(&lines.join("\n")) {
            self.mode = n;
            self.note = None;
            self.save();
        }
        self.redraw();
    }

    fn menu(&mut self, title: &str, lines: &[String]) -> Option<usize> {
        let w = lines.iter().map(|l| l.chars().count()).max().unwrap_or(20).max(title.len()) + 4;
        let mut p = self.lower_right(w as u16, lines.len() as u16 + 2);
        p.pane.border_fg = Some(109);
        p.modal(&lines.join("\n"))
    }

    fn popup(&mut self, text: &str, w: u16) {
        let (cols, rows) = Crust::terminal_size();
        let h = (text.lines().count() as u16 + 2).min(rows.saturating_sub(2));
        let w = w.min(cols.saturating_sub(4));
        let mut p = Pane::new(cols.saturating_sub(w).max(1), rows.saturating_sub(h).max(1), w, h, 252, 235);
        p.border = true;
        p.scroll = false;
        p.set_text(text);
        p.full_refresh();
        let _ = Input::getchr(None);
        self.redraw();
    }

    /// A bordered popup in the lower right, the way the rest of the suite
    /// does it.
    fn lower_right(&self, w: u16, h: u16) -> crust::Popup {
        let (cols, rows) = Crust::terminal_size();
        let w = w.min(cols.saturating_sub(4));
        let h = h.min(rows.saturating_sub(2));
        let mut p = crust::Popup::new(cols.saturating_sub(w).max(1), rows.saturating_sub(h).max(1), w, h, 252, 235);
        p.pane.scroll = true;
        p
    }

    fn redraw(&mut self) {
        Crust::clear_screen();
        self.shown = Default::default();
        self.rows.clear();
        self.dirty = true;
    }

    // ── Drawing ────────────────────────────────────────────────────────

    fn render(&mut self) {
        let (cols, rows) = Crust::terminal_size();
        let (cols, rows) = (cols as usize, rows as usize);
        let bw = cols.saturating_sub(INFO_W).max(24);
        let bh = rows.saturating_sub(1).max(6);

        let head = self.header(cols);
        if head != self.shown[0] {
            let mut p = Pane::new(1, 1, cols as u16, 1, 255, 236);
            p.wrap = false;
            p.scroll = false;
            p.set_text(&head);
            p.refresh();
            self.shown[0] = head;
        }

        if self.dirty {
            let lines = self.bench_lines(bw, bh);
            if self.rows.len() != lines.len() { self.rows = vec![String::new(); lines.len()]; }
            let mut frame = String::new();
            for (i, line) in lines.into_iter().enumerate() {
                if line != self.rows[i] {
                    frame.push_str(&Cursor::at(1, i as u16 + 2));
                    frame.push_str(&line);
                    self.rows[i] = line;
                }
            }
            if !frame.is_empty() {
                use std::io::Write;
                let mut out = std::io::stdout();
                let _ = out.write_all(frame.as_bytes());
                let _ = out.flush();
            }
            self.dirty = false;
        }

        let info = self.info(cols - bw, bh);
        if info != self.shown[1] {
            let mut p = Pane::new(bw as u16 + 1, 2, (cols - bw) as u16, bh as u16, 252, 234);
            p.wrap = true;
            p.scroll = false;
            p.set_text(&info);
            p.refresh();
            self.shown[1] = info;
        }
    }

    fn header(&self, cols: usize) -> String {
        let title = if self.mode == 0 {
            "Free play".to_string()
        } else {
            let tick = if self.done[self.mode] { " ✓" } else { "" };
            format!("Experiment {}: {}{tick}", self.mode, LESSONS[self.mode - 1].title)
        };
        let mut facts = vec![style::bold("alchemy"), title];
        if self.screen { facts.push(style::fg("screen up", 46)); }
        else { facts.push(style::fg("screen down", 208)); }
        let keys = "SPACE shelf   h burner   t flame   w wash   x tasks   ? help   q quit";
        let version = format!("v{}", env!("CARGO_PKG_VERSION"));
        let width = |s: &str| crust::strip_ansi(s).chars().count();
        let right = width(keys) + 3 + width(&version) + 1;
        let mut left = format!(" {}", facts.join("   "));
        while facts.len() > 1 && width(&left) + right + 2 > cols {
            facts.pop();
            left = format!(" {}", facts.join("   "));
        }
        let pad = cols.saturating_sub(width(&left) + right).max(1);
        format!("{left}{}{keys}   {} ", " ".repeat(pad), style::fg(&version, 245))
    }

    /// The bench, one string per row, ANSI and all.
    fn bench_lines(&self, bw: usize, bh: usize) -> Vec<String> {
        if let Some(_) = self.blind { return self.blind_lines(bw, bh); }
        let block = VAP + GH + 3;
        let top = (bh.saturating_sub(block)) / 2;
        let left = (bw.saturating_sub(SLOT * GLASSES)) / 2;
        let mut out = vec![String::new(); bh];

        for (i, v) in self.bench.iter().enumerate() {
            let cells = self.glass_cells(v, i);
            for (r, row) in cells.into_iter().enumerate() {
                let y = top + r;
                if y >= bh { break; }
                if out[y].is_empty() { out[y] = " ".repeat(left); }
                out[y].push_str(&row);
            }
        }
        // A flame test burns above the bench.
        if let Some((rgb, _, _)) = self.flame {
            let torch = "▲▲▲";
            let y = top.saturating_sub(1);
            let pad = left + self.cur * SLOT + SLOT / 2 - 1;
            out[y] = format!("{}{}", " ".repeat(pad), style::rgb(torch, Some(rgb), None, "b"));
        }
        // The line under the bench: what just happened.
        if let Some(note) = &self.note {
            let y = (top + block + 1).min(bh - 1);
            let text: String = note.chars().take(bw.saturating_sub(4)).collect();
            out[y] = format!("  {}", style::fg(&text, 250));
        }
        for row in out.iter_mut() {
            let w = crust::strip_ansi(row).chars().count();
            if w < bw { row.push_str(&" ".repeat(bw - w)); }
        }
        out
    }

    /// Eighteen minutes, told in four seconds.
    fn blind_lines(&self, bw: usize, bh: usize) -> Vec<String> {
        let mut out = vec![" ".repeat(bw); bh];
        let since = self.blind.map(|t| t.elapsed().as_secs_f64()).unwrap_or(0.0);
        let words: &[&str] = if since < 0.6 {
            &[]
        } else if since < 2.0 {
            &["It smelled of burnt hair."]
        } else {
            &["It smelled of burnt hair.", "", "You cannot see. It is completely black."]
        };
        for (n, w) in words.iter().enumerate() {
            if w.is_empty() { continue; }
            let y = bh / 2 + n;
            let x = (bw.saturating_sub(w.chars().count())) / 2;
            if y < bh { out[y] = format!("{}{}{}", " ".repeat(x), style::fg(w, 240), " ".repeat(bw - x - w.chars().count())); }
        }
        if since < 0.25 {
            let flash = self.bench.iter().map(|v| v.flare_rgb).next().unwrap_or((255, 255, 255));
            for row in out.iter_mut() { *row = style::rgb(&" ".repeat(bw), None, Some(flash), ""); }
        }
        out
    }

    /// One glass drawn as a column of rows, each already padded to SLOT.
    fn glass_cells(&self, v: &Vessel, i: usize) -> Vec<String> {
        let mut rows: Vec<String> = Vec::with_capacity(VAP + GH + 3);
        let picked = i == self.cur;
        let wall = if picked { 231u8 } else { 246u8 };
        let inner = |r: usize| -> usize {
            match v.glass {
                Glass::Tube => 4,
                Glass::Beaker => 9,
                Glass::Flask => if r < 3 { 3 } else { (3 + (r - 2) * 2).min(11) },
            }
        };
        let widest = (0..GH).map(inner).max().unwrap_or(4);
        let pad = |w: usize| (SLOT - w) / 2;

        // Vapour over the glass.
        for r in 0..VAP {
            let line = match v.vapour() {
                Some(rgb) if v.volume() > 0.1 || v.temp > 40.0 => {
                    let w = inner(0);
                    let puff: String = (0..w).map(|c| if self.noise(i, r, c) % 3 == 0 { '░' } else { ' ' }).collect();
                    format!("{}{}", " ".repeat(pad(w)), style::rgb(&puff, Some(rgb), None, ""))
                }
                _ => String::new(),
            };
            rows.push(fit(&line, SLOT));
        }

        // The glass itself.
        let liquid = v.liquid_rgb();
        // The bottom row is the rim, so the inside of the glass is one
        // row shorter than the glass.
        let deep = GH - 1;
        let level = (v.level() * deep as f64).round() as usize;
        let grit = v.settled().map(|(rgb, amount)| (rgb, ((amount * 0.15).ceil() as usize).clamp(1, deep)));
        let hot = v.temp > 60.0;
        for r in 0..GH {
            let w = inner(r);
            let from_bottom = (deep - 1).saturating_sub(r);
            let wet = r < deep && from_bottom < level;
            let in_grit = r < deep && grit.map(|(_, h)| from_bottom < h).unwrap_or(false);
            let body: String = if v.flare > 0.0 {
                style::rgb(&" ".repeat(w), None, Some(v.flare_rgb), "")
            } else if in_grit {
                let (rgb, _) = grit.unwrap();
                style::rgb(&"▒".repeat(w), Some(pale(rgb)), Some(rgb), "")
            } else if wet {
                let bg = liquid.unwrap_or((198, 222, 236));
                let face: String = (0..w).map(|c| {
                    if v.fizz > 1e-3 && self.noise(i, r, c) % 7 == 0 { '∘' } else { ' ' }
                }).collect();
                style::rgb(&face, Some(pale(bg)), Some(bg), "")
            } else {
                " ".repeat(w)
            };
            let (l, rr) = if r == GH - 1 {
                (style::rgb("╰", Some(rgb_of(wall)), None, ""), style::rgb("╯", Some(rgb_of(wall)), None, ""))
            } else {
                (style::rgb("│", Some(rgb_of(wall)), None, ""), style::rgb("│", Some(rgb_of(wall)), None, ""))
            };
            let floor = if r == GH - 1 {
                style::rgb(&"─".repeat(w), Some(rgb_of(wall)), None, "")
            } else { body };
            rows.push(fit(&format!("{}{l}{floor}{rr}", " ".repeat(pad(w + 2))), SLOT));
        }

        // The burner.
        let flame = if v.burner {
            let shapes = ["ʌʌʌ", "ʌVʌ", "Vʌ V"];
            let s = shapes[(self.frame / 2) as usize % shapes.len()];
            let c = if hot { (255, 170, 40) } else { (120, 160, 240) };
            format!("{}{}", " ".repeat(pad(3)), style::rgb(s, Some(c), None, "b"))
        } else { String::new() };
        rows.push(fit(&flame, SLOT));

        // The label.
        let name = format!("{} {}", i + 1, v.glass.name());
        let label = if picked { style::fb(&format!(" {name} "), 232, 250) } else { style::fg(&name, 244) };
        rows.push(fit(&format!("{}{label}", " ".repeat(pad(name.chars().count() + 2))), SLOT));
        let _ = widest;
        rows.push(" ".repeat(SLOT));
        rows
    }

    /// A cheap repeatable scatter, so bubbles and vapour move without a
    /// random number generator and without allocating.
    fn noise(&self, glass: usize, row: usize, col: usize) -> u64 {
        let mut x = self.frame / 2 + (glass as u64) * 7919 + (row as u64) * 104729 + (col as u64) * 15485863;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        x
    }

    /// The panel on the right: what is in the glass you have picked.
    fn info(&self, w: usize, h: usize) -> String {
        let v = &self.bench[self.cur];
        let mut out: Vec<String> = Vec::new();
        let rule = "─".repeat(w.saturating_sub(2));

        out.push(format!(" {}", style::bold(&format!("{} {}", self.cur + 1, v.glass.name()))));
        let ph = v.ph();
        out.push(format!(" {:.0} mL   {:.0} °C   pH {:.1} {}", v.volume(), v.temp, ph, ph_word(ph)));
        out.push(format!(" {}", style::fg(&rule, 238)));

        if v.is_empty() {
            out.push(style::fg(" empty. SPACE opens the shelf.", 245));
        } else {
            for (sp, mmol) in v.contents().into_iter().take(9) {
                let colour = sp.rgb.map(|c| style::rgb("■", Some(c), None, "")).unwrap_or_else(|| " ".into());
                out.push(format!(" {colour} {:<17}{:>7} {:>4.1} mmol", trim(sp.name, 17), sp.formula, mmol));
            }
            if v.volume() > 0.1 {
                out.push(format!("   {:<17}{:>7} {:>4.0} mL", "water", "H₂O", v.volume()));
            }
        }

        out.push(String::new());
        out.push(format!(" {}", style::bold("What you see")));
        for line in self.seen(v) { out.push(format!("   {line}")); }

        if let Some((_, what, _)) = self.flame {
            out.push(String::new());
            out.push(format!(" {}", style::bold("In the flame")));
            out.push(format!("   {what}"));
        }

        if let Some((sp, _)) = v.contents().first() {
            out.push(String::new());
            for l in wrap(sp.note, w.saturating_sub(3)) { out.push(format!(" {}", style::fg(&l, 109))); }
        }

        if self.mode > 0 {
            let lesson = &LESSONS[self.mode - 1];
            let used = out.len() + 5;
            out.push(String::new());
            out.push(format!(" {}", style::fg(&rule, 238)));
            out.push(format!(" {}", style::bold(lesson.goal)));
            let room = h.saturating_sub(used);
            for l in wrap(lesson.hint, w.saturating_sub(3)).into_iter().take(room) {
                out.push(format!(" {}", style::fg(&l, 245)));
            }
        }
        out.join("\n")
    }

    /// What the glass looks like, said in plain words.
    fn seen(&self, v: &Vessel) -> Vec<String> {
        let mut out = Vec::new();
        if v.is_empty() { return vec![style::fg("nothing at all", 245)]; }
        if v.blue() > 0.15 {
            out.push(style::rgb("blue black, from iodine on starch", Some((110, 130, 235)), None, ""));
        } else if let Some(rgb) = v.liquid_rgb() {
            let word = colour_word(rgb);
            out.push(format!("{} {}", style::rgb("■", Some(rgb), None, ""), format!("a {word} liquid")));
        } else if v.volume() > 0.1 {
            out.push("a clear liquid".to_string());
        }
        if let Some((rgb, amount)) = v.settled() {
            let (what, grain) = if v.volume() > 0.1 {
                ("settled on the bottom", if amount > 8.0 { "powder" } else { "grains" })
            } else {
                ("in the glass", if amount > 8.0 { "crystals" } else { "grains" })
            };
            out.push(format!("{} {} {grain} {what}", style::rgb("■", Some(rgb), None, ""), colour_word(rgb)));
        }
        if v.fizz > 1e-3 { out.push("bubbles rising".to_string()); }
        if let Some(rgb) = v.vapour() { out.push(format!("{} vapour over the liquid", style::rgb("■", Some(rgb), None, ""))); }
        if v.temp > 60.0 { out.push(style::fg("too hot to hold", 208)); }
        else if v.temp > 30.0 { out.push("warm to the touch".to_string()); }
        if out.is_empty() { out.push(style::fg("nothing worth reporting", 245)); }
        out
    }
}

fn dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
    PathBuf::from(home).join(".alchemy")
}

/// Pad or cut a row of ANSI to exactly `w` printable columns.
fn fit(s: &str, w: usize) -> String {
    let have = crust::strip_ansi(s).chars().count();
    if have >= w { s.to_string() } else { format!("{s}{}", " ".repeat(w - have)) }
}

fn trim(s: &str, w: usize) -> String {
    if s.chars().count() <= w { s.to_string() } else { s.chars().take(w - 1).chain("…".chars()).collect() }
}

fn wrap(text: &str, w: usize) -> Vec<String> {
    let mut out = Vec::new();
    let mut line = String::new();
    for word in text.split_whitespace() {
        if !line.is_empty() && line.chars().count() + 1 + word.chars().count() > w {
            out.push(std::mem::take(&mut line));
        }
        if !line.is_empty() { line.push(' '); }
        line.push_str(word);
    }
    if !line.is_empty() { out.push(line); }
    out
}

/// A lighter shade of a colour, for bubbles on top of it.
fn pale(c: Rgb) -> Rgb {
    let up = |x: u8| (x as u16 + 90).min(255) as u8;
    (up(c.0), up(c.1), up(c.2))
}

fn rgb_of(x: u8) -> Rgb { let v = if x > 250 { 235 } else { 150 }; (v, v, v) }

/// The everyday name for a colour, so the panel can say what you see.
fn colour_word(c: Rgb) -> &'static str {
    let (r, g, b) = (c.0 as i32, c.1 as i32, c.2 as i32);
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    if max - min < 25 { return if max > 200 { "white" } else if max > 90 { "grey" } else { "black" }; }
    if b == max && r < b - 40 && g < b - 40 { return if b > 180 { "blue" } else { "deep blue" }; }
    if g == max && b > r { return "blue green"; }
    if g == max { return "green"; }
    if r == max && g > 150 && b < 120 { return "yellow"; }
    if r == max && g > 90 && b < 90 { return "orange"; }
    if r == max && b > 120 { return "purple"; }
    if r == max && g < 110 { return if r > 190 { "red" } else { "brown" }; }
    "pale"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_shelf_item_is_a_real_substance() {
        for (_, items) in SHELF {
            for i in *items {
                assert!(species(i.key).is_some(), "no such substance: {}", i.key);
                assert!(i.mmol > 0.0 || i.ml > 0.0, "{} pours nothing", i.label);
            }
        }
    }

    #[test]
    fn the_shelf_can_pass_the_experiments() {
        // Every substance an experiment needs has to be reachable.
        let on_shelf = |k: &str| SHELF.iter().any(|(_, items)| items.iter().any(|i| i.key == k));
        for k in ["CuSO4", "starch", "I2", "HCl", "NaOH", "AgNO3", "NaCl", "PbNO3", "KI",
                  "CaOH2", "NaHCO3", "Cu", "H2O2", "H2SO4", "Na2S2O3", "KIO3", "MA", "MnSO4"] {
            assert!(on_shelf(k), "{k} is not on the shelf");
        }
    }

    #[test]
    fn colours_get_everyday_names() {
        assert_eq!(colour_word((0, 120, 205)), "blue");
        assert_eq!(colour_word((240, 200, 40)), "yellow");
        assert_eq!(colour_word((250, 250, 250)), "white");
        assert_eq!(colour_word((150, 70, 25)), "brown");
        assert_eq!(colour_word((40, 225, 180)), "blue green");
    }

    #[test]
    fn a_row_is_padded_to_the_slot() {
        let painted = style::rgb("ab", Some((1, 2, 3)), None, "");
        assert_eq!(crust::strip_ansi(&fit(&painted, 6)).chars().count(), 6);
    }

    #[test]
    fn a_fresh_bench_has_nothing_to_do() {
        let app = App::new();
        assert!(!app.working(), "an empty bench must not tick");
    }
}
