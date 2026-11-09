#![cfg_attr(windows, windows_subsystem = "windows")]

use eframe::egui::{self, Align2, Color32, FontId, Pos2, Stroke, Vec2};
use std::f32::consts::TAU;
use std::path::PathBuf;

const SPIN_SECONDS: f64 = 4.5;
const COLORS: [Color32; 4] = [
    Color32::from_rgb(240, 190, 40),
    Color32::from_rgb(215, 30, 50),
    Color32::from_rgb(50, 100, 220),
    Color32::from_rgb(40, 165, 70),
];

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("SARK Loting")
            .with_inner_size([1280.0, 800.0]),
        ..Default::default()
    };
    eframe::run_native(
        "SARK Loting",
        options,
        Box::new(|_| {
            let mut app = App::default();
            // Allow opening a CSV via command line / "Open with".
            if let Some(path) = std::env::args().nth(1) {
                app.load_path(PathBuf::from(path));
            }
            Ok(Box::new(app))
        }),
    )
}

struct Participant {
    cols: Vec<String>,
    number: Option<u32>,
    preset: bool,
    preset_input: String,
}

struct Spin {
    start_time: f64,
    from: f32,
    to: f32,
    number: u32,
}

#[derive(Default)]
struct App {
    file: Option<PathBuf>,
    header: Vec<String>,
    delimiter: u8,
    num_col: usize,
    participants: Vec<Participant>,
    pool_input: String,
    started: bool,
    pool: Vec<u32>,
    /// (participant index, number came from pool)
    history: Vec<(usize, bool)>,
    manual_input: String,
    rotation: f32,
    spin: Option<Spin>,
    message: String,
}

impl App {
    fn current(&self) -> Option<usize> {
        self.participants.iter().position(|p| p.number.is_none())
    }

    fn display_name(&self, p: &Participant) -> String {
        // Navigator / Bestuurder layout of the SARK file; otherwise join all columns.
        if p.cols.len() >= 5 && self.num_col == 0 {
            format!(
                "{} {}  /  {} {}",
                p.cols[2], p.cols[1], p.cols[4], p.cols[3]
            )
        } else {
            p.cols
                .iter()
                .enumerate()
                .filter(|(i, _)| *i != self.num_col)
                .map(|(_, c)| c.as_str())
                .collect::<Vec<_>>()
                .join(" ")
        }
    }

    fn open_csv(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("CSV", &["csv"])
            .pick_file()
        else {
            return;
        };
        self.load_path(path);
    }

    fn load_path(&mut self, path: PathBuf) {
        match load_csv(&path) {
            Ok((header, delimiter, rows)) => {
                let num_col = header
                    .iter()
                    .position(|h| {
                        let h = h.to_lowercase();
                        h.contains("eq") || h.contains("nummer")
                    })
                    .unwrap_or(0);
                self.participants = rows
                    .into_iter()
                    .map(|mut cols| {
                        cols.resize(header.len().max(cols.len()), String::new());
                        let preset_input = cols[num_col].trim().to_string();
                        Participant {
                            cols,
                            number: None,
                            preset: false,
                            preset_input,
                        }
                    })
                    .collect();
                self.header = header;
                self.delimiter = delimiter;
                self.num_col = num_col;
                self.file = Some(path);
                self.started = false;
                self.pool.clear();
                self.history.clear();
                self.spin = None;
                if self.pool_input.is_empty() {
                    self.pool_input = format!("1-{}", self.participants.len());
                }
                self.message = format!("{} deelnemers geladen.", self.participants.len());
            }
            Err(e) => self.message = format!("Fout bij openen: {e}"),
        }
    }

    fn start(&mut self) {
        let pool = match parse_numbers(&self.pool_input) {
            Ok(p) => p,
            Err(e) => {
                self.message = e;
                return;
            }
        };
        let mut presets: Vec<Option<u32>> = Vec::new();
        for p in &self.participants {
            let t = p.preset_input.trim();
            if t.is_empty() {
                presets.push(None);
            } else {
                match t.parse::<u32>() {
                    Ok(n) => presets.push(Some(n)),
                    Err(_) => {
                        self.message = format!("Ongeldig vast nummer: '{t}'");
                        return;
                    }
                }
            }
        }
        let mut seen = std::collections::HashSet::new();
        for n in presets.iter().flatten() {
            if !seen.insert(*n) {
                self.message = format!("Vast nummer {n} is meer dan één keer toegewezen.");
                return;
            }
        }
        let remaining: Vec<u32> = pool.into_iter().filter(|n| !seen.contains(n)).collect();
        let to_draw = presets.iter().filter(|p| p.is_none()).count();
        if remaining.len() < to_draw {
            self.message = format!(
                "Te weinig nummers: {} nummers voor {} deelnemers.",
                remaining.len(),
                to_draw
            );
            return;
        }
        for (p, n) in self.participants.iter_mut().zip(presets) {
            p.number = n;
            p.preset = n.is_some();
        }
        self.pool = remaining;
        self.history.clear();
        self.started = true;
        self.message = format!(
            "Loting gestart: {} vaste nummers, {} te loten.",
            seen.len(),
            to_draw
        );
    }

    fn reset(&mut self) {
        for p in &mut self.participants {
            p.number = None;
            p.preset = false;
        }
        self.started = false;
        self.pool.clear();
        self.history.clear();
        self.spin = None;
        self.message = "Loting gereset.".into();
    }

    fn assign(&mut self, number: u32) {
        let Some(idx) = self.current() else { return };
        if self.participants.iter().any(|p| p.number == Some(number)) {
            self.message = format!("Nummer {number} is al toegewezen.");
            return;
        }
        let from_pool = if let Some(pos) = self.pool.iter().position(|&n| n == number) {
            self.pool.remove(pos);
            true
        } else {
            false
        };
        self.participants[idx].number = Some(number);
        self.history.push((idx, from_pool));
        self.message = format!(
            "Nummer {number} → {}",
            self.display_name(&self.participants[idx])
        );
    }

    fn undo(&mut self) {
        if let Some((idx, from_pool)) = self.history.pop()
            && let Some(n) = self.participants[idx].number.take()
        {
            if from_pool {
                self.pool.push(n);
                self.pool.sort_unstable();
            }
            self.message = format!("Nummer {n} teruggezet.");
        }
    }

    fn start_spin(&mut self, now: f64) {
        if self.pool.is_empty() || self.current().is_none() {
            return;
        }
        let i = fastrand::usize(..self.pool.len());
        let seg = TAU / self.pool.len() as f32;
        // Pointer sits at angle 0 (right side). Land somewhere inside segment i.
        let offset = seg * (0.15 + 0.7 * fastrand::f32());
        let target = -(i as f32 * seg + offset);
        let delta = (target - self.rotation).rem_euclid(TAU);
        self.spin = Some(Spin {
            start_time: now,
            from: self.rotation,
            to: self.rotation + 6.0 * TAU + delta,
            number: self.pool[i],
        });
    }

    fn export(&mut self) {
        let default_name = self
            .file
            .as_ref()
            .and_then(|f| f.file_stem())
            .map(|s| format!("{}-uitslag.csv", s.to_string_lossy()))
            .unwrap_or_else(|| "loting-uitslag.csv".into());
        let Some(path) = rfd::FileDialog::new()
            .add_filter("CSV", &["csv"])
            .set_file_name(default_name)
            .save_file()
        else {
            return;
        };
        let rows: Vec<Vec<String>> = self
            .participants
            .iter()
            .map(|p| {
                let mut cols = p.cols.clone();
                cols[self.num_col] = p.number.map(|n| n.to_string()).unwrap_or_default();
                cols
            })
            .collect();
        match save_csv(&path, &self.header, self.delimiter, &rows) {
            Ok(()) => self.message = format!("Geëxporteerd naar {}", path.display()),
            Err(e) => self.message = format!("Fout bij exporteren: {e}"),
        }
    }

    fn wheel(&mut self, ui: &mut egui::Ui) {
        let size = ui
            .available_width()
            .min(ui.available_height() - 150.0)
            .max(200.0);
        let (rect, _) = ui.allocate_exact_size(Vec2::splat(size), egui::Sense::hover());
        let painter = ui.painter_at(rect.expand(20.0));
        let c = rect.center();
        let r = size / 2.0 - 10.0;

        let n = self.pool.len();
        if n == 0 {
            painter.circle_filled(c, r, Color32::DARK_GRAY);
        } else {
            let seg = TAU / n as f32;
            let steps = (64 / n).max(2);
            for (i, num) in self.pool.iter().enumerate() {
                let a0 = self.rotation + i as f32 * seg;
                let mut pts = vec![c];
                for s in 0..=steps {
                    let a = a0 + seg * s as f32 / steps as f32;
                    pts.push(c + Vec2::angled(a) * r);
                }
                let color = if n % 4 == 1 && i == n - 1 && n > 1 {
                    COLORS[1] // avoid two equal neighbours
                } else {
                    COLORS[i % 4]
                };
                painter.add(egui::Shape::convex_polygon(
                    pts,
                    color,
                    Stroke::new(1.0, Color32::from_black_alpha(80)),
                ));
                let mid = a0 + seg / 2.0;
                let font = (r * 0.12).min(r * seg * 0.55).clamp(10.0, 64.0);
                painter.text(
                    c + Vec2::angled(mid) * r * 0.78,
                    Align2::CENTER_CENTER,
                    num.to_string(),
                    FontId::proportional(font),
                    Color32::BLACK,
                );
            }
        }
        painter.circle_filled(c, r * 0.22, Color32::WHITE);
        painter.circle_stroke(
            c,
            r * 0.22,
            Stroke::new(3.0, Color32::from_rgb(215, 30, 50)),
        );
        painter.text(
            c,
            Align2::CENTER_CENTER,
            "SARK",
            FontId::proportional(r * 0.12),
            Color32::from_rgb(215, 30, 50),
        );
        // Pointer on the right, pointing at the wheel.
        let tip = Pos2::new(c.x + r - 8.0, c.y);
        painter.add(egui::Shape::convex_polygon(
            vec![
                tip,
                Pos2::new(tip.x + 34.0, c.y - 16.0),
                Pos2::new(tip.x + 34.0, c.y + 16.0),
            ],
            Color32::from_rgb(200, 20, 30),
            Stroke::new(2.0, Color32::WHITE),
        ));
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let now = ui.input(|i| i.time);

        // Advance the spin animation.
        if let Some(spin) = &self.spin {
            let t = ((now - spin.start_time) / SPIN_SECONDS).clamp(0.0, 1.0) as f32;
            let ease = 1.0 - (1.0 - t).powi(4);
            self.rotation = spin.from + (spin.to - spin.from) * ease;
            if t >= 1.0 {
                let n = spin.number;
                self.rotation = spin.to.rem_euclid(TAU);
                self.spin = None;
                self.assign(n);
            } else {
                ui.ctx().request_repaint();
            }
        }
        let spinning = self.spin.is_some();

        egui::Panel::top("top").show(ui, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.heading("SARK Loting");
                ui.separator();
                if ui
                    .add_enabled(!spinning, egui::Button::new("📂 CSV openen"))
                    .clicked()
                {
                    self.open_csv();
                }
                let has_data = !self.participants.is_empty();
                if ui
                    .add_enabled(has_data && !spinning, egui::Button::new("💾 Exporteren"))
                    .clicked()
                {
                    self.export();
                }
                if self.started
                    && ui
                        .add_enabled(!spinning, egui::Button::new("⟲ Opnieuw beginnen"))
                        .clicked()
                {
                    self.reset();
                }
                if let Some(f) = &self.file {
                    ui.label(f.file_name().unwrap_or_default().to_string_lossy());
                }
            });
            if !self.message.is_empty() {
                ui.label(&self.message);
            }
            ui.add_space(4.0);
        });

        egui::Panel::left("deelnemers")
            .default_size(520.0)
            .show(ui, |ui| self.participant_list(ui));

        egui::CentralPanel::default().show(ui, |ui| {
            if self.participants.is_empty() {
                ui.centered_and_justified(|ui| {
                    ui.heading("Open eerst een CSV-bestand met deelnemers.")
                });
                return;
            }
            if !self.started {
                self.setup_panel(ui);
                return;
            }
            ui.vertical_centered(|ui| {
                match self.current() {
                    Some(idx) => {
                        ui.label("Nu aan de beurt:");
                        ui.heading(
                            egui::RichText::new(self.display_name(&self.participants[idx]))
                                .size(26.0)
                                .strong(),
                        );
                    }
                    None => {
                        ui.heading(egui::RichText::new("Loting voltooid!").size(26.0).strong());
                        ui.label("Vergeet niet te exporteren.");
                    }
                }
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    let can_draw = !spinning && self.current().is_some();
                    if ui
                        .add_enabled(
                            can_draw && !self.pool.is_empty(),
                            egui::Button::new(egui::RichText::new("🎲 Draai").size(22.0)),
                        )
                        .clicked()
                    {
                        self.start_spin(now);
                    }
                    ui.separator();
                    ui.label("Handmatig nummer:");
                    let resp = ui.add_enabled(
                        can_draw,
                        egui::TextEdit::singleline(&mut self.manual_input).desired_width(60.0),
                    );
                    let enter = resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                    if (ui
                        .add_enabled(can_draw, egui::Button::new("Toewijzen"))
                        .clicked()
                        || enter)
                        && can_draw
                    {
                        match self.manual_input.trim().parse::<u32>() {
                            Ok(n) => {
                                self.assign(n);
                                self.manual_input.clear();
                            }
                            Err(_) => self.message = "Voer een geldig nummer in.".into(),
                        }
                    }
                    ui.separator();
                    if ui
                        .add_enabled(
                            !spinning && !self.history.is_empty(),
                            egui::Button::new("↶ Ongedaan maken"),
                        )
                        .clicked()
                    {
                        self.undo();
                    }
                    ui.label(format!("{} nummers over", self.pool.len()));
                });
                if !spinning && let Some(&(idx, _)) = self.history.last() {
                    let p = &self.participants[idx];
                    ui.label(
                        egui::RichText::new(format!(
                            "Laatst getrokken: {}  →  {}",
                            p.number.unwrap_or_default(),
                            self.display_name(p)
                        ))
                        .size(20.0)
                        .color(Color32::from_rgb(240, 190, 40)),
                    );
                }
                ui.add_space(10.0);
                self.wheel(ui);
            });
        });
    }
}

impl App {
    fn participant_list(&mut self, ui: &mut egui::Ui) {
        let current = if self.started { self.current() } else { None };
        let started = self.started;
        let names: Vec<String> = self
            .participants
            .iter()
            .map(|p| self.display_name(p))
            .collect();
        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("grid")
                .striped(true)
                .num_columns(3)
                .show(ui, |ui| {
                    ui.strong("#");
                    ui.strong("Deelnemers (navigator / bestuurder)");
                    ui.strong(if started { "Nummer" } else { "Vast nr." });
                    ui.end_row();
                    for (i, p) in self.participants.iter_mut().enumerate() {
                        ui.label(format!("{}", i + 1));
                        let mut name = egui::RichText::new(&names[i]);
                        if current == Some(i) {
                            name = name.strong().color(Color32::from_rgb(240, 190, 40));
                        }
                        ui.label(name);
                        if started {
                            match p.number {
                                Some(n) => {
                                    let mut t = egui::RichText::new(n.to_string()).strong();
                                    if p.preset {
                                        t = t.color(Color32::from_rgb(120, 170, 255));
                                    }
                                    ui.label(t).on_hover_text(if p.preset {
                                        "Vast nummer"
                                    } else {
                                        "Geloot"
                                    });
                                }
                                None => {
                                    ui.label("–");
                                }
                            }
                        } else {
                            ui.add(
                                egui::TextEdit::singleline(&mut p.preset_input).desired_width(50.0),
                            );
                        }
                        ui.end_row();
                    }
                });
        });
    }

    fn setup_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("Voorbereiding");
        ui.add_space(8.0);
        ui.label("Nummers in de loting (bijv. 1-57 of 1-40, 45, 50-60):");
        ui.add(egui::TextEdit::singleline(&mut self.pool_input).desired_width(400.0));
        match parse_numbers(&self.pool_input) {
            Ok(p) => ui.label(format!("{} nummers", p.len())),
            Err(e) => ui.colored_label(Color32::RED, e),
        };
        ui.add_space(8.0);
        ui.label(
            "Vaste nummers: vul in de lijst links bij 'Vast nr.' een nummer in voor deelnemers \
             met een vooraf bepaald nummer. Deze nummers worden uit de loting gehaald en deze \
             deelnemers worden overgeslagen.",
        );
        ui.add_space(8.0);
        ui.label("Daarna wordt in de volgorde van de lijst geloot.");
        ui.add_space(12.0);
        if ui
            .button(egui::RichText::new("▶ Start loting").size(22.0))
            .clicked()
        {
            self.start();
        }
    }
}

/// Parses "1-57, 60, 70-72" into a sorted, deduplicated list.
fn parse_numbers(s: &str) -> Result<Vec<u32>, String> {
    let mut out = Vec::new();
    for part in s
        .split([',', ';', ' '])
        .map(str::trim)
        .filter(|p| !p.is_empty())
    {
        let err = || format!("Ongeldig deel: '{part}'");
        if let Some((a, b)) = part.split_once('-') {
            let a: u32 = a.trim().parse().map_err(|_| err())?;
            let b: u32 = b.trim().parse().map_err(|_| err())?;
            if a > b || b - a > 100_000 {
                return Err(err());
            }
            out.extend(a..=b);
        } else {
            out.push(part.parse().map_err(|_| err())?);
        }
    }
    out.sort_unstable();
    out.dedup();
    if out.is_empty() {
        return Err("Geen nummers opgegeven.".into());
    }
    Ok(out)
}

type CsvData = (Vec<String>, u8, Vec<Vec<String>>);

fn load_csv(path: &PathBuf) -> Result<CsvData, String> {
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    let bytes = bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(&bytes);
    // Accept UTF-8, fall back to Windows-1252/Latin-1 (Excel exports).
    let text = match std::str::from_utf8(bytes) {
        Ok(s) => s.to_string(),
        Err(_) => bytes.iter().map(|&b| b as char).collect(),
    };
    let first = text.lines().next().unwrap_or("");
    let delimiter = if first.matches(';').count() > first.matches(',').count() {
        b';'
    } else {
        b','
    };
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(delimiter)
        .flexible(true)
        .from_reader(text.as_bytes());
    let header: Vec<String> = rdr
        .headers()
        .map_err(|e| e.to_string())?
        .iter()
        .map(|s| s.trim().to_string())
        .collect();
    let mut rows = Vec::new();
    for rec in rdr.records() {
        let rec = rec.map_err(|e| e.to_string())?;
        let cols: Vec<String> = rec.iter().map(|s| s.trim().to_string()).collect();
        if cols.iter().all(|c| c.is_empty()) {
            continue;
        }
        rows.push(cols);
    }
    if rows.is_empty() {
        return Err("Geen deelnemers gevonden in het bestand.".into());
    }
    Ok((header, delimiter, rows))
}

fn save_csv(
    path: &PathBuf,
    header: &[String],
    delimiter: u8,
    rows: &[Vec<String>],
) -> Result<(), String> {
    let mut buf = b"\xEF\xBB\xBF".to_vec(); // BOM so Excel shows accents correctly
    {
        let mut w = csv::WriterBuilder::new()
            .delimiter(delimiter)
            .flexible(true)
            .from_writer(&mut buf);
        w.write_record(header).map_err(|e| e.to_string())?;
        for r in rows {
            w.write_record(r).map_err(|e| e.to_string())?;
        }
        w.flush().map_err(|e| e.to_string())?;
    }
    std::fs::write(path, buf).map_err(|e| e.to_string())
}
