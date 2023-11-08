use std::{fmt::Display, str::FromStr};

use eframe::egui;
use num_traits::{FromPrimitive, clamp_max};
use num_derive::FromPrimitive;

/*
 * There are 3 basic modes of operation:
 * - change one of the gears, keeping the ratio fixed
 *      -> the other gear will be adapted, the actual ratio can diverge from the given ratio
 *      due to rounding to whole teeth numbers
 * - change one of the gears, keeping the other gear fixed
 *      - the ratio will be adapted
 * - change the ratio, keeping one of the gears fixed
 *      - again, the actual ratio will move in steps
 */

#[derive(PartialEq, FromPrimitive, Debug, Clone, Copy)]
enum Column {
    Left  = 0b001,
    Ratio = 0b010,
    Right = 0b100,
}

impl Column {
    // get the missing 3rd column for 2 columns. c1 and c2 may not be equal.
    fn get_missing(c1: Column, c2: Column) -> Column {
        assert_ne!(c1, c2);
        let mut i = c1 as u32 | c2 as u32;
        i = (!i) & 0b111;
        let c: Column = FromPrimitive::from_u32(i).unwrap();
        c
    }

    // the long and cumbersome version:
    //fn get_missing(c1: Column, c2: Column) -> Column {
        //match c1 {
            //Column::Left => match c2 {
                //Column::Ratio => Column::Right,
                //_             => Column::Ratio,
            //},
            //Column::Ratio => match c2 {
                //Column::Left => Column::Right,
                //_            => Column::Left,
            //},
            //Column::Right => match c2 {
                //Column::Left => Column::Ratio,
                //_            => Column::Left,
            //},
        //}
    //}
    // another alternative would be looping through the values
}

struct SideVars {
    //column: Column,
    teeth: u32,
    t_str: String,
}

impl SideVars {
    fn new(column: Column, teeth: u32) -> SideVars {
        SideVars{
            //column,
            teeth,
            t_str: String::from(teeth.to_string()),
        }
    }
}

pub struct RitzelApp {
    left: SideVars,
    right: SideVars,
    given_ratio: f32,
    actual_ratio: f32,
    ar_str: String,
    gr_str: String,
    locked_column: Column,
}

#[derive(Clone, Copy, Default)]
struct NumberSpinnerState {
    offset: f32,
    rect_max: egui::Pos2,
}

fn number_spinner<T>(ui: &mut egui::Ui, value: &mut T, val_str: &mut String, interactive: bool, step: T, min_value: T, max_value: T, precision: usize, uiid: i32) -> bool
where
    // aaaah just give me a sane number type
    T: num_traits::NumAssign + PartialOrd + Display + FromPrimitive + FromStr + Copy
{
    let mut changed = false;
    let myid = egui::Id::new(34234 + uiid);
    //let mut state: NumberSpinnerState = ui.ctx.ge
    let mut state: NumberSpinnerState = ui.ctx().data_mut(|d| d.get_temp(myid)).unwrap_or_default();
    ui.vertical(|ui| {
        // handle scrolling and dragging.
        // handling drags needs to be done before adding other ui elements to not steal their
        // input
        if interactive {
            let mut delta = 0.0;
            let mut urect = ui.min_rect();
            urect.max = state.rect_max;

            // scrolling
            ui.input(|i| {
                if let Some(pos) = i.pointer.latest_pos() {
                    if urect.contains(pos){
                        delta = i.scroll_delta.y;
                    }
                }
            });

            // dragging
            let resp = ui.interact(urect, myid, egui::Sense::drag());
            if resp.dragged() {
                //println!("Dragged by: {:?}", resp.drag_delta());
                delta = resp.drag_delta().y;
            }

            if delta != 0.0 {
                state.offset += delta;
                //println!("offset: {}", state.offset);
                if state.offset > 20.0 {
                    state.offset = 0.0;
                    *value = clamp_max(*value + step, max_value);
                    changed = true;
                } else if state.offset < -20.0 {
                    state.offset = 0.0;
                    //*value = clamp_min(*value - step, min_value); // but avoid uint underflows
                    //(-0.00001 to fix float precision problems, otherwise ratio only goes to 0.2)
                    *value = if *value >= min_value + step - T::from_f32(0.00001).unwrap() {
                        *value - step
                    } else {
                        *value
                    };
                    changed = true;
                }
                ui.ctx().data_mut(|d| d.insert_temp(myid, state));
                // number changed from scroll/drag, so we need to update the text field
                if changed {
                    *val_str = format!("{0:.1$}", *value, precision).to_owned();
                }
            }
        }

        let te = egui::TextEdit::singleline(val_str)
            .interactive(interactive)
            .desired_width(80.0);

        ui.label(egui::RichText::new(format!("{1:.0$}", precision, clamp_max(*value + step * T::from_f32(2.0).unwrap(), max_value))).weak());
        ui.label(egui::RichText::new(format!("{1:.0$}", precision, clamp_max(*value + step, max_value))).weak());
        let te_response = ui.add(te);
        ui.label(egui::RichText::new(
                if *value  >= min_value + step - T::from_f32(0.00001).unwrap() {
                    format!("{1:.0$}", precision, *value - step)
                } else {
                    "".to_owned()
                }).weak());
        ui.label(egui::RichText::new(
                if *value >= min_value + step + step - T::from_f32(0.00001).unwrap() {
                    format!("{1:.0$}", precision, *value - step - step)
                } else {
                    "".to_owned()
                }).weak());

        if state.rect_max != ui.min_rect().max {
            state.rect_max = ui.min_rect().max;
            ui.ctx().data_mut(|d| d.insert_temp(myid, state));
        }

        // if enter is pressed and the entered string is no valid number, reset it
        if te_response.lost_focus() {
            if let Err(_) = val_str.parse::<T>() {
                *val_str = format!("{0:.1$}", *value, precision).to_owned();
            }
        }
        if te_response.changed() {
            if let Ok(x) = val_str.parse::<T>() {
                *value = x;
                changed = true;
            }
        }
    });
    changed
}


impl RitzelApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        RitzelApp {
            left: SideVars::new(Column::Left, 10),
            right: SideVars::new(Column::Right, 15),
            given_ratio: 1.5,
            actual_ratio: 1.5,
            ar_str: String::from(1.5.to_string()),
            gr_str: String::from(1.5.to_string()),
            locked_column: Column::Ratio,
        }
    }

    // left gear is the motor, right gear the wheel.
    // ratio is theeth on wheel / teeth on motor.
    fn compute_ratio(&mut self) {
        self.actual_ratio = self.right.teeth as f32 / self.left.teeth as f32;
        self.ar_str = String::from(format!("{:.3}", self.actual_ratio));
    }

    fn compute_l_teeth(&mut self) {
        let lt = self.right.teeth as f32 / self.given_ratio;
        self.left.teeth = lt.round() as u32;
        self.left.t_str = String::from(self.left.teeth.to_string());
        // the actual ratio may not be the exact ratio due to the rounding
        self.compute_ratio();
    }

    fn compute_r_teeth(&mut self) {
        let rt = self.left.teeth as f32 * self.given_ratio;
        self.right.teeth = rt.round() as u32;
        self.right.t_str = String::from(self.right.teeth.to_string());
        // the actual ratio may not be the exact ratio due to the rounding
        self.compute_ratio();
    }

    // recomputes the value that is not fixed and not changed
    fn recompute_from(&mut self, column: Column) {
        let c = Column::get_missing(column, self.locked_column);
        match c {
            Column::Left => self.compute_l_teeth(),
            Column::Ratio => self.compute_ratio(),
            Column::Right => self.compute_r_teeth(),
        };
    }

    fn gear_column(&mut self, ui: &mut egui::Ui, column: Column) {
        ui.vertical(|ui| {
            let vars = match column {
                Column::Left => &mut self.left,
                _            => &mut self.right,
            };
            let changed = number_spinner(ui, &mut vars.teeth, &mut vars.t_str, column != self.locked_column, 1, 1, 100000, 0, column as i32);
            if changed {
                self.recompute_from(column);
            }
            ui.selectable_value(&mut self.locked_column, column, "locked");
        });
    }

    fn ratio_column(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            // given ratio row
            ui.horizontal(|ui| {
                ui.label("Given Ratio: ");
                let changed = number_spinner(ui, &mut self.given_ratio, &mut self.gr_str, self.locked_column != Column::Ratio, 0.1, 0.1, 100.0, 2, Column::Ratio as i32);
                if changed {
                    self.recompute_from(Column::Ratio);
                }
            });

            // actual ratio row
            ui.horizontal(|ui| {
                ui.label("Actual Ratio: ");
                ui.label(&self.ar_str);
            });

            ui.selectable_value(&mut self.locked_column, Column::Ratio, "locked");
        });
    }

}

impl eframe::App for RitzelApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Gear Ratio Calculator");
            ui.horizontal(|ui| {
                // labels
                ui.horizontal(|ui| {
                    self.gear_column(ui, Column::Left);
                    self.ratio_column(ui);
                    self.gear_column(ui, Column::Right);
                });
            });
        });
    }
}

