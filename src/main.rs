use core::fmt;
use std::{
    collections::BTreeSet,
    ops::{Mul, MulAssign},
    str::FromStr,
};

use cubing::alg::{Alg, Move};

mod moveseq;

use itertools::Itertools;
use moveseq::MoveSeq;

const ZOOM: f32 = 1.5;

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

    regions: BTreeSet<Region>,
    grips: BTreeSet<Grip>,
}
impl App {
    pub fn new(cc: &eframe::CreationContext) -> Self {
        cc.egui_ctx.set_zoom_factor(ZOOM);
        let mut ret = Self {
            alg_str: "[R, U] [U2, R]".to_string(),
            ..Default::default()
        };
        ret.update_moves_from_alg_str();
        ret
    }

    fn update_moves_from_alg_str(&mut self) {
        let result = Alg::from_str(&self.alg_str);
        self.alg_is_valid = result.is_ok();
        if let Ok(alg) = result {
            self.moves = MoveSeq::from_alg(&alg);

            let inverse_moves = MoveSeq::from_alg(&alg.invert());
            self.regions = BTreeSet::from_iter([Region::default()]);
            for m in inverse_moves.iter() {
                self.regions = std::mem::take(&mut self.regions)
                    .into_iter()
                    .flat_map(|r| r.do_move(m.clone()))
                    .collect();
            }

            self.grips = self
                .regions
                .iter()
                .flat_map(|r| itertools::chain(&r.include, &r.exclude))
                .cloned()
                .collect()
        }
    }
}
impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if ui.text_edit_singleline(&mut self.alg_str).changed() {
                self.update_moves_from_alg_str();
            }
            if !self.alg_is_valid {
                ui.colored_label(ui.visuals().error_fg_color, "error!");
            } else {
                ui.label(self.moves.to_string());
            }
            ui.separator();

            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
            ui.columns(2, |uis| {
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
            write!(f, "{grip_name} @ ({transform})")
        }
    }
}
impl MulAssign<Move> for Grip {
    fn mul_assign(&mut self, rhs: Move) {
        self.transform.push_back(rhs);
        if self.transform.len() == 1 {
            self.simplify_first();
        }
    }
}
impl Mul<Move> for Grip {
    type Output = Self;

    fn mul(mut self, rhs: Move) -> Self::Output {
        self *= rhs;
        self
    }
}
impl Grip {
    pub fn new(grip_name: String) -> Self {
        Self {
            grip_name,
            transform: MoveSeq::new(),
        }
    }
    fn simplify_first(&mut self) {
        self.transform.pop_front_if_eq(&self.grip_name);
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
    #[must_use]
    pub fn do_move(self, m: Move) -> Vec<Region> {
        match self.has_grip(Grip::new(m.quantum.family.clone())) {
            Some(true) => {
                vec![self.do_move_unchecked(m)]
            }
            Some(false) => vec![self],
            None => {
                let mut excluded = self.clone();
                excluded.exclude.insert(Grip::new(m.quantum.family.clone()));
                let mut included = self.do_move_unchecked(m.clone());
                included.include.insert(Grip::new(m.quantum.family.clone()));
                vec![excluded, included]
            }
        }
    }
    #[must_use]
    fn do_move_unchecked(mut self, m: Move) -> Self {
        for set in [&mut self.include, &mut self.exclude] {
            *set = std::mem::take(set)
                .into_iter()
                .map(|mut g| {
                    g *= m.clone();
                    g
                })
                .collect();
        }

        self
    }
}
