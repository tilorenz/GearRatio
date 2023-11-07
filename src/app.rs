use eframe::egui;
use num_traits::FromPrimitive;
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
    column: Column,
    teeth: u32,
    t_str: String,
}

impl SideVars {
    fn new(column: Column, teeth: u32) -> SideVars {
        SideVars{
            column,
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


fn number_spinner(ui: &mut egui::Ui, value: &mut f32, val_str: &mut String, interactive: bool, step: f32, min_value: f32, max_value: f32, precision: usize, uiid: i32) -> bool
{
    let mut changed = false;
    ui.vertical(|ui| {
        let te = egui::TextEdit::singleline(val_str)
            .interactive(interactive)
            .desired_width(80.0);

        ui.label(egui::RichText::new(format!("{1:.0$}", precision, (*value + step * 2.0).min(max_value))).weak());
        ui.label(egui::RichText::new(format!("{1:.0$}", precision, (*value + step).min(max_value))).weak());
        let response = ui.add(te);
        ui.label(egui::RichText::new(format!("{1:.0$}", precision, (*value - step).max(min_value))).weak());
        ui.label(egui::RichText::new(format!("{1:.0$}", precision, (*value - step * 2.0).max(min_value))).weak());

        // if enter is pressed and the entered string is no valid number, reset it
        if response.lost_focus() {
            if let Err(_) = val_str.parse::<f32>() {
                *val_str = String::from(value.to_string());
            }
        }
        if response.changed() {
            if let Ok(x) = val_str.parse::<f32>() {
                *value = x;
                changed = true;
            }
        }

        // handle scrolling
        if interactive {
            // TODO accumulate delta and only do the scroll if it surpassed some value.
            // ideally, also animate that.
            // currently, this is broken for laptop touchpads
            let mut delta = 0.0;
            ui.input(|i| {
                if let Some(pos) = i.pointer.latest_pos() {
                    if ui.min_rect().contains(pos){
                        delta = i.scroll_delta.y;
                    }
                }
            });
            if delta > 0.0 {
                *value = (*value + step).min(max_value);
                changed = true;
            } else if delta < 0.0 {
                *value = (*value - step).max(min_value);
                changed = true;
            }

            let resp = ui.interact(ui.min_rect(), egui::Id::new(34234 + uiid), egui::Sense::drag());
            if resp.dragged() {
                println!("Dragged by: {:?}", resp.drag_delta());
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
            let mut float_proxy = vars.teeth as f32;
            let changed = number_spinner(ui, &mut float_proxy, &mut vars.t_str, column != self.locked_column, 1.0, 1.0, f32::INFINITY, 0, column as i32);
            if changed {
                vars.teeth = float_proxy.round() as u32;
                vars.t_str = String::from(vars.teeth.to_string());
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
                    self.gr_str = String::from(format!("{:.2}", self.given_ratio));
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


