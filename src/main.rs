use core::fmt;
use std::{
    collections::{BTreeMap, BTreeSet},
    str::FromStr,
};

use cubing::alg::{Alg, Move};

mod moveseq;

use itertools::Itertools;
use moveseq::MoveSeq;

const ZOOM: f32 = 1.5;

const DEFAULT_ALG: &str = "[R, U] [U2, R]";
const DEFAULT_RELATIONS: &str = "\
    U = F * R\n\
    R = U * F\n\
    F = R * U\n\
    L = F * U\n\
    F = U * L\n\
";

fn main() -> eframe::Result {
    eframe::run_native(
        "Grippy",
        eframe::NativeOptions::default(),
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}

#[derive(Default)]
struct App {
    alg_str: String,
    alg_is_valid: bool,
    moves: MoveSeq,

    relations_str: String,
    relations_str_error: Option<String>,
    relations: Vec<Relation>,

    regions: BTreeSet<Region>,
    grips: BTreeSet<Grip>,
    results: BTreeMap<MoveSeq, BTreeMap<Vec<bool>, Vec<Region>>>,
}
impl App {
    pub fn new(cc: &eframe::CreationContext) -> Self {
        cc.egui_ctx.set_zoom_factor(ZOOM);
        let mut ret = Self {
            alg_str: DEFAULT_ALG.to_string(),
            relations_str: DEFAULT_RELATIONS.to_string(),
            ..Default::default()
        };
        ret.recompute_everything();
        ret
    }

    fn recompute_everything(&mut self) {
        self.relations_str_error = None;

        // Parse relations
        self.relations = vec![];
        for line in self.relations_str.lines() {
            let line = line.trim();
            if !line.is_empty() {
                let Some((lhs, rhs)) = line.split_once("=") else {
                    self.relations_str_error =
                        Some(format!("relation line {line:?} is missing '='"));
                    break;
                };
                let new_grip_name = lhs.trim().to_owned();
                if let Err(e) = validate_grip_name(&new_grip_name) {
                    self.relations_str_error = Some(e);
                    break;
                }

                let Some((rhs1, rhs2)) =
                    Option::or_else(rhs.trim().split_once('*'), || rhs.trim().split_once('×'))
                else {
                    self.relations_str_error =
                        Some(format!("relation line {line:?} is missing '*' or '×'"));
                    break;
                };
                let old_grip_name = rhs1.trim().to_owned();
                if let Err(e) = validate_grip_name(&old_grip_name) {
                    self.relations_str_error = Some(e);
                    break;
                }

                match Alg::from_str(rhs2) {
                    Ok(alg) => {
                        // Add inverse relation
                        self.relations.push(Relation {
                            new_grip_name: old_grip_name.clone(),
                            grip_to_replace: Grip {
                                grip_name: new_grip_name.clone(),
                                transform: MoveSeq::from_alg(&alg.invert()),
                            },
                        });
                        // Add original relation
                        self.relations.push(Relation {
                            new_grip_name,
                            grip_to_replace: Grip {
                                grip_name: old_grip_name,
                                transform: MoveSeq::from_alg(&alg),
                            },
                        });
                    }
                    Err(e) => self.relations_str_error = Some(e.to_string()),
                }
            }
        }

        // Parse alg
        let alg_result = Alg::from_str(&self.alg_str);
        self.alg_is_valid = alg_result.is_ok();

        self.grips.clear();
        self.regions.clear();
        self.results.clear();

        if !self.alg_is_valid || self.relations_str_error.is_some() {
            return;
        }

        if let Ok(alg) = alg_result {
            self.moves = MoveSeq::from_alg(&alg);

            let inverse_moves = MoveSeq::from_alg(&alg.invert());
            self.regions = BTreeSet::from_iter([Region::default()]);
            for m in inverse_moves.iter() {
                self.regions = std::mem::take(&mut self.regions)
                    .into_iter()
                    .flat_map(|r| r.do_move(m.clone(), &self.relations))
                    .flatten()
                    .collect();
            }

            self.grips = self
                .regions
                .iter()
                .flat_map(|r| itertools::chain(&r.include, &r.exclude))
                .cloned()
                .collect();

            for region in &self.regions {
                let mut move_seq = MoveSeq::new();
                let mut move_mask = vec![];
                let mut r = region.clone();
                for m in self.moves.iter() {
                    let [not_affected, affected] = r.do_move(m.clone(), &self.relations);
                    move_mask.push(affected.is_some());
                    if affected.is_some() {
                        move_seq.push_back(m.clone());
                    }
                    r = affected.or(not_affected).unwrap();
                }
                self.results
                    .entry(move_seq)
                    .or_default()
                    .entry(move_mask)
                    .or_default()
                    .push(region.clone());
            }
        }
    }
}
impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.columns(2, |uis| {
                {
                    let ui = &mut uis[0];
                    ui.label("Algorithm:");
                    if ui.text_edit_singleline(&mut self.alg_str).changed() {
                        self.recompute_everything();
                    }
                    match !self.alg_is_valid {
                        true => ui.colored_label(ui.visuals().error_fg_color, "error!"),
                        false => ui.label(self.moves.to_string()),
                    };
                }
                {
                    let ui = &mut uis[1];
                    ui.label("Relations:");
                    if ui.text_edit_multiline(&mut self.relations_str).changed() {
                        self.recompute_everything();
                    }
                    match &self.relations_str_error {
                        Some(e) => ui.colored_label(ui.visuals().error_fg_color, e),
                        None => ui.label(""),
                    };
                }
            });
            ui.separator();

            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
            ui.columns(3, |uis| {
                egui::ScrollArea::new([true; 2])
                    .auto_shrink(false)
                    .id_salt("regions")
                    .show(&mut uis[0], |ui| {
                        ui.heading(format!("Grips ({})", self.grips.len()));
                        for g in &self.grips {
                            ui.label(g.to_string());
                        }
                    });

                egui::ScrollArea::new([true; 2])
                    .auto_shrink(false)
                    .id_salt("grips")
                    .show(&mut uis[1], |ui| {
                        ui.heading(format!("Regions ({})", self.regions.len()));
                        for r in &self.regions {
                            ui.label(r.to_string());
                        }
                    });

                egui::ScrollArea::new([true; 2])
                    .auto_shrink(false)
                    .id_salt("results")
                    .show(&mut uis[2], |ui| {
                        ui.heading(format!("Results ({})", self.results.len()));
                        for (move_seq, regions_by_move_seq) in &self.results {
                            let move_seq_str = if move_seq.is_empty() {
                                "(empty)".to_string()
                            } else {
                                move_seq.to_string()
                            };
                            ui.label(format!("Net move sequence: {move_seq_str}"));
                            for (move_mask, regions) in regions_by_move_seq {
                                let mut job = egui::text::LayoutJob::default();
                                let get_text_format = |color| {
                                    egui::TextFormat::simple(
                                        egui::FontId::proportional(13.0),
                                        color,
                                    )
                                };
                                let mut is_first = true;
                                job.append("    ", 0.0, get_text_format(ui.visuals().text_color()));
                                for (m, include) in self.moves.iter().zip(move_mask) {
                                    let pre = if is_first { "" } else { " " };
                                    is_first = false;
                                    let color = ui
                                        .visuals()
                                        .text_color()
                                        .gamma_multiply(if *include { 1.25 } else { 0.5 });
                                    job.append(&format!("{pre}{m}"), 0.0, get_text_format(color));
                                }
                                ui.label(job);
                                for r in regions {
                                    ui.label(format!("        {r}"));
                                }
                            }
                        }
                    });
            });
        });
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Grip {
    grip_name: String,
    transform: MoveSeq,
}
impl fmt::Display for Grip {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self {
            grip_name,
            transform,
        } = self;

        if self.transform.is_empty() {
            write!(f, "{grip_name}")
        } else {
            write!(f, "{grip_name} × ({transform})")
        }
    }
}
impl Grip {
    pub fn new(grip_name: String) -> Self {
        Self {
            grip_name,
            transform: MoveSeq::new(),
        }
    }
    #[must_use]
    pub fn do_move(mut self, m: Move, relations: &[Relation]) -> Self {
        self.transform.push_back(m);

        // Grip is not affected by its own move
        if self.transform.len() == 1 {
            self.transform.pop_front_if_fam(&self.grip_name);
        }

        // Apply relations
        for r in relations {
            if r.grip_to_replace.grip_name == self.grip_name
                && self
                    .transform
                    .pop_front_if_matches(&r.grip_to_replace.transform)
            {
                self.grip_name = r.new_grip_name.clone();
            }
        }

        self
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Region {
    include: BTreeSet<Grip>,
    exclude: BTreeSet<Grip>,
}
impl fmt::Display for Region {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = itertools::chain(
            self.include.iter().map(|s| format!("{s}")),
            self.exclude.iter().map(|s| format!("!{s}")),
        )
        .join(", ");

        write!(f, "{{{s}}}")
    }
}
impl Region {
    pub fn has_grip(&self, grip: Grip) -> Option<bool> {
        if self.include.contains(&grip) {
            Some(true)
        } else if self.exclude.contains(&grip) {
            Some(false)
        } else {
            None
        }
    }
    /// returns `[not_affected, affected]`
    #[must_use]
    pub fn do_move(self, m: Move, relations: &[Relation]) -> [Option<Region>; 2] {
        match self.has_grip(Grip::new(m.quantum.family.clone())) {
            Some(false) => [Some(self), None],
            Some(true) => [None, Some(self.do_move_unchecked(m, relations))],
            None => {
                let mut excluded = self.clone();
                excluded.exclude.insert(Grip::new(m.quantum.family.clone()));
                let mut included = self.do_move_unchecked(m.clone(), relations);
                included.include.insert(Grip::new(m.quantum.family.clone()));
                [Some(excluded), Some(included)]
            }
        }
    }
    #[must_use]
    fn do_move_unchecked(mut self, m: Move, relations: &[Relation]) -> Self {
        for set in [&mut self.include, &mut self.exclude] {
            *set = std::mem::take(set)
                .into_iter()
                .map(|g| g.do_move(m.clone(), relations))
                .collect();
        }

        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Relation {
    pub new_grip_name: String,
    pub grip_to_replace: Grip,
}

fn validate_grip_name(s: &str) -> Result<(), String> {
    if s.chars().all(|c| c.is_alphabetic() || c == '_') {
        Ok(())
    } else {
        Err(format!("invalid grip {s:?}"))
    }
}
