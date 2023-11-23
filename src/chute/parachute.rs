#![allow(unused)]



extern crate nalgebra as na;
use std::default;

use dxf;
use dxf::Drawing;
use dxf::entities::*;
use eframe::egui;
use egui_plot;
use evalexpr::ContextWithMutableVariables;
use uom::unit;
use crate::chute::parachute;
use crate::chute::ui;
use crate::chute::geometry;

use nalgebra::Vector2;

use std::f64::consts::PI;

use uom::si::{self, length};

use super::geometry::ToPoints;

// Represents a band. Can contain multiple geometries representing the cross section, but is symmetrical and has a fixed number of gores
#[derive(Clone)]
pub struct PolygonalChuteSection {
}

// Represents a chute section that can be bent into a perfectly circular disk/cone/cylinder. Only a straight line allowed
#[derive(Clone)]
pub struct CircularChuteSection {
    line: geometry::Line,
    expressions: [String; 4]
}

impl Default for CircularChuteSection {
    fn default() -> Self {
        Self {
            line: geometry::Line { begin: Vector2::new(0.0, 0.0), end: Vector2::new(1.0, 0.0)},
            expressions: ["0".to_string(), "0".to_string(), "1.0".to_string(), "0.0".to_string()].into()
        }
    }
}

#[derive(Clone)]
pub enum ChuteSectionType {
    Polygonal(PolygonalChuteSection),
    Circular(CircularChuteSection)
}

#[derive(Clone)]
pub struct ChuteSection {
    section_type: ChuteSectionType,
    gores: u16,
    fabric: FabricSelector,
    seam_allowance: (f64, f64, f64, f64), // Right, top, left, bottom
}

impl ChuteSection {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame, use_imperial: bool, evaluator_context: &evalexpr::HashMapContext, index_id: u16) {
        ui.label("Fabric:");
        self.fabric.ui(ui, frame, use_imperial, index_id);
        
        ui.label("Number of gores:").on_hover_text("Number of parachute gores. Typically between 6 and 24");
        ui::integer_edit_field(ui, &mut self.gores);

        let eval = |expr: &str| evalexpr::eval_number_with_context(expr, evaluator_context).unwrap_or(0.0);
        
        match &mut self.section_type {
            ChuteSectionType::Circular(sec) => {
                ui.label("Start point:");

                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut sec.expressions[0]);
                    ui.text_edit_singleline(&mut sec.expressions[1]);
                });

                ui.label("End point:");

                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut sec.expressions[2]);
                    ui.text_edit_singleline(&mut sec.expressions[3]);
                });

                sec.line.begin.x = eval(&sec.expressions[0]);
                sec.line.begin.y = eval(&sec.expressions[1]);
                sec.line.end.x = eval(&sec.expressions[2]);
                sec.line.end.y = eval(&sec.expressions[3]);

            },
            ChuteSectionType::Polygonal(sec) => {
                todo!()
            }
        }
    }

    fn new_circular() -> Self {
        Self { section_type: ChuteSectionType::Circular(CircularChuteSection::default()), gores: 8, fabric: FabricSelector::new(), seam_allowance: (0.01, 0.01, 0.01, 0.01) }
    }

    fn get_cross_section(&self, resolution: u32) -> geometry::Points {
        // to 2D section
        match &self.section_type {
            ChuteSectionType::Circular(circ) => {
                circ.line.to_points(resolution)
            },
            ChuteSectionType::Polygonal(poly) => {
                todo!()
            }
        }
    }

    fn get_gore() {
    }

    fn get_3d_model() {

    }
}

impl geometry::ToPoints for ChuteSection {
    fn to_points(&self, resolution: u32) -> geometry::Points {
        self.get_cross_section(resolution)
    }
}

// Standard unit combinations for input sliders
#[derive(PartialEq, Clone, Default)]
pub enum StandardUnit {
    #[default] UnitLess, // Can be used if other SI units are needed
    MeterFoot,
    MillimeterInch,
    Radian,
    Degree,
}

impl StandardUnit {
    pub fn get_options() -> Vec<Self> {
        vec![
            Self::UnitLess,
            Self::MeterFoot,
            Self::MillimeterInch,
            Self::Radian,
            Self::Degree,
        ]
    }

    pub fn get_general_name(&self) -> String {
        match self {
            Self::UnitLess => "unitless".into(),
            Self::MeterFoot => "m | ft".into(),
            Self::MillimeterInch => "mm | in".into(),
            Self::Radian => "rad".into(),
            Self::Degree => "deg".into(),
        }
    }
}

// Represents a slider that can be used as input
#[derive(PartialEq, Clone)]
pub struct InputValue {
    pub id: String, // Unique identifier
    pub description: String, // Short description of what it does
    pub value: f64,
    pub unit: StandardUnit,
    pub range: std::ops::RangeInclusive<f64>, // Range, given in SI base unit
}

// These are parameters that are computed using the InputValues. 
// They are evaluated sequentially and may refer to previous computed ParameterValue variables.
#[derive(PartialEq, Clone)]
pub struct ParameterValue {
    pub id: String,
    pub expression: String, // Mathematical expression. Value not needed since it's saved in the context
    pub display_unit: StandardUnit, // Only for display purposes. 
}


// Parachute designer interface, implements the relevant UI drawing functions
#[derive(Clone)]
pub struct ChuteDesigner {
    name: String,
    gores: u16,
    diameter: f64,

    fabric: FabricSelector,
    instructions: Vec<String>,
    use_global_seam_allowance: bool,
    global_seam_allowance: f64,

    // just for testing
    test_color: [f32; 3],

    input_values: Vec<InputValue>, // Each needs a name, value, range (in m or deg).
    parameter_values: Vec<ParameterValue>, // always in SI units

    chute_sections: Vec<ChuteSection>,

    evaluator_context: evalexpr::HashMapContext, // evaluator that handles variables etc. Note: stored value always in SI base unit
}

impl PartialEq for ChuteDesigner {
    fn eq(&self, other: &Self) -> bool {
        if self.name.ne(&other.name) { false }
        else if self.gores.ne(&other.gores) { false }
        else if self.diameter.ne(&other.diameter) { false }
        else if self.fabric.ne(&other.fabric) { false }
        else if self.instructions.ne(&other.instructions) { false }
        else if self.use_global_seam_allowance.ne(&other.use_global_seam_allowance) { false }
        else if self.test_color.ne(&other.test_color) { false }
        else if self.input_values.ne(&other.input_values) { false }
        else if self.parameter_values.ne(&other.parameter_values) { false }
        else { true }
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}


unit! {
    system: uom::si;
    quantity: uom::si::length;

    @unitless: 1.0; "-", "-", "-";
}


impl ChuteDesigner {
    pub fn options_ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame, use_imperial: bool) {
        
        // Number of gores
        ui::integer_edit_field(ui, &mut self.gores);

        ui.checkbox(&mut self.use_global_seam_allowance, "Use global seam allowance");

        ui::dimension_field(ui, &mut self.diameter, use_imperial, 0.0..=10.0);
        ui::number_edit_field(ui, &mut self.diameter);

        ui.add_enabled(self.use_global_seam_allowance, egui::Slider::new::<f64>(&mut self.global_seam_allowance, 0.0..=5.0));
        ui.add_enabled(self.use_global_seam_allowance, egui::Checkbox::new(&mut false, "Cut corners of seam allowance"));

        //egui::widgets::color_picker::color_edit_button_rgb(ui, &mut self.test_color);

        ui.horizontal(|ui| {
            ui.label("Name:");
            ui.text_edit_singleline(&mut self.name)
        });

        for (idx, input_value) in self.input_values.iter_mut().enumerate() {
            ui.label(&input_value.id);
            ui.horizontal(|ui| {
                ui.label("Value:");
                
                match &input_value.unit {
                    StandardUnit::UnitLess => ui::length_slider(ui, &mut input_value.value, use_imperial, input_value.range.clone(), &unitless, &unitless),
                    StandardUnit::MeterFoot => ui::length_slider(ui, &mut input_value.value, use_imperial, input_value.range.clone(), &length::meter, &length::foot),
                    StandardUnit::MillimeterInch => ui::length_slider(ui, &mut input_value.value, use_imperial, input_value.range.clone(), &length::millimeter, &length::inch),
                    StandardUnit::Degree => ui::length_slider(ui, &mut input_value.value, use_imperial, input_value.range.clone(), &si::angle::degree, &si::angle::degree),
                    StandardUnit::Radian => ui::length_slider(ui, &mut input_value.value, use_imperial, input_value.range.clone(), &si::angle::radian, &si::angle::radian),
                };

                if !input_value.description.is_empty() {
                    ui.button("❓").on_hover_text_at_pointer(&input_value.description);
                }

            });

            ui.separator();
        }
    }

    pub fn instructions_ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        ui.label(egui::RichText::new("Instructions").font(egui::FontId::proportional(20.0)));
        if ui.button("Add step").clicked() {
            self.instructions.push("Step".to_owned());
        }

        let mut to_delete: Option<usize> = None;

        for (num, step) in self.instructions.iter().enumerate() {
            if ui.selectable_label(false, format!("{}: {}", num + 1, step)).clicked() {
                to_delete = Some(num);
            }
        }

        if let Some(delete_idx) = to_delete {
            self.instructions.remove(delete_idx);
        }
    }

    pub fn draw_cross_section(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        // Generate points
        /* 
        let cross = geometry::Points::from_vec((0..80).map(|ang| {
            na::Vector2::new((ang as f64 * PI/180.0).cos() * self.diameter * 0.5,(ang as f64 * PI/180.0).sin() * 0.7 * self.diameter * 0.5)
        }).collect());

        let cross1 = geometry::Points::from_vec((0..80).map(|ang| {
            na::Vector2::new(-(ang as f64 * PI/180.0).cos() * self.diameter * 0.5,(ang as f64 * PI/180.0).sin() * 0.7 * self.diameter * 0.5)
        }).collect());

        */

        //let lines = vec![cross, cross1];
        let mut lines = self.get_cross_section();
        lines.append(&mut self.get_cross_section().iter().map(|p| p.mirror_x()).collect());

        self.equal_aspect_plot(ui, frame, &lines, Some(1));
    }

    pub fn equal_aspect_plot(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame, data: &Vec<geometry::Points>, highlighted: Option<u16>) {
        let mut lines = vec![];

        for (idx,line) in data.iter().enumerate() {
            
            let pts: egui_plot::PlotPoints = line.points.iter().map(|pt| [pt.x, pt.y]).collect();
            let this_line = egui_plot::Line::new(pts).width(2.0).highlight(highlighted == Some(idx as u16));
            
            lines.push(this_line);
        }

        egui_plot::Plot::new("cross_section").height(300.0).data_aspect(1.0).view_aspect(1.5).auto_bounds_x().auto_bounds_y().show(ui, |plot_ui| {
            for line in lines {
                plot_ui.line(line);
            }
        });
    }

    fn has_id_error(id: &String) -> Option<String> {
        if id.contains(char::is_whitespace) {
            return Some("Error: ID cannot contain whitespace characters".into());
        }
        else if id.len() == 0 {
            return Some("Error: ID cannot be empty".into());
        }
        else if !id.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Some("Error: ID must be alphanumeric".into());
        }
        else if !id.chars().next().is_some_and(char::is_alphabetic) {
            return Some("Error: First letter must be alphabetic".into());
        }
        else {
            return None;
        };
    }

    fn default_vars() -> Vec<(String, f64)> {
        vec![
            ("m".into(), 1.0),
            ("mm".into(), 0.001),
            ("yd".into(), 0.9144),
            ("ft".into(), 0.3048),
            ("inch".into(), 0.0254),
            ("rad".into(), 1.0),
            ("pi".into(), PI),
            ("e".into(), core::f64::consts::E),
            ("deg".into(), PI / 180.0)]
    }

    pub fn geometry_ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame, use_imperial: bool) {
        // geometry stuff

        self.evaluator_context.clear_variables();
        for (name, val) in ChuteDesigner::default_vars() {
            self.evaluator_context.set_value(name, evalexpr::Value::Float(val)).unwrap();
        }
        

        ui.columns(2, |columns| {
            let mut ui = &mut columns[0];

            ui.heading("Input variables:");
            ui.separator();
    
            if ui.button("Add input").clicked() {
                self.input_values.push(InputValue { description: "".into(), id: format!("input{}", self.input_values.len()+1), range: 0.0..=10.0, unit: StandardUnit::MeterFoot, value: 0.0 })
            }
    
            let mut to_delete: Option<usize> = None;
            let mut to_move: Option<(usize, bool)> = None;
            let num_parameters = self.input_values.len();
    
            for (idx, input_value) in self.input_values.iter_mut().enumerate() {
                
                ui.horizontal(|ui| {
                    if ui.button("❌").clicked() {
                        to_delete = Some(idx);
                    };
                    if ui.add_enabled(idx != 0, egui::Button::new("⬆")).clicked() {
                        to_move = Some((idx, true));
                    }
                    if ui.add_enabled(idx < num_parameters - 1, egui::Button::new("⬇")).clicked() {
                        to_move = Some((idx, false));
                    };
                });
                
                ui.horizontal(|ui| {
                    ui.label("ID:");
                    ui.text_edit_singleline(&mut input_value.id);
                });
                ui.horizontal(|ui| {
                    ui.label("Description");
                    ui.text_edit_singleline(&mut input_value.description);
                });
    
                egui::ComboBox::from_id_source(&input_value.id).width(200.0)
                .selected_text(input_value.unit.get_general_name())
                .show_ui(ui, |ui| {
                    for (_idx, option) in StandardUnit::get_options().iter().enumerate() {
                        ui.selectable_value(&mut input_value.unit, option.clone(), option.get_general_name());
                    }
                });
    
    
                ui.horizontal(|ui| {
                    ui.label("Value:");
                    
                    match &input_value.unit {
                        StandardUnit::UnitLess => ui::length_slider(ui, &mut input_value.value, use_imperial, input_value.range.clone(), &unitless, &unitless),
                        StandardUnit::MeterFoot => ui::length_slider(ui, &mut input_value.value, use_imperial, input_value.range.clone(), &length::meter, &length::foot),
                        StandardUnit::MillimeterInch => ui::length_slider(ui, &mut input_value.value, use_imperial, input_value.range.clone(), &length::millimeter, &length::inch),
                        StandardUnit::Degree => ui::length_slider(ui, &mut input_value.value, use_imperial, input_value.range.clone(), &si::angle::degree, &si::angle::degree),
                        StandardUnit::Radian => ui::length_slider(ui, &mut input_value.value, use_imperial, input_value.range.clone(), &si::angle::radian, &si::angle::radian),
                    }                
                });
    
                // TODO: Add unit conversion selection
                let mut start = *input_value.range.start();
                let mut end = *input_value.range.end();
    
                ui.horizontal(|ui| {
                    ui.label("Range: ");
                    ui::number_edit_field(ui, &mut start);
                    ui.label("to");
                    ui::number_edit_field(ui, &mut end);
                });
                input_value.range = start ..= end;
    
                if evalexpr::Context::get_value(&self.evaluator_context, &input_value.id).is_some() {
                    ui.label("Error: Identifier already used");
                }
                else if let Some(msg) = ChuteDesigner::has_id_error(&input_value.id) {
                    ui.label(msg);
                } else {
                    self.evaluator_context.set_value(input_value.id.clone(), evalexpr::Value::Float(input_value.value)).unwrap_or_else(|_| {
                        ui.label("Unable to save value...");
                    })
                }
                ui.separator();
            }
    
            if let Some((idx, direction)) = to_move {
                if idx > 0 && direction {
                    // Swap upwards
                    self.input_values.swap(idx, idx-1);
                } else if idx < (self.input_values.len() - 1) && !direction{
                    self.input_values.swap(idx, idx+1);
                }
            }
    
            
            if let Some(delete_idx) = to_delete {
                self.input_values.remove(delete_idx);
            }



            ui.heading("Computed parameters");
            if ui.button("Add parameter").clicked() {
                self.parameter_values.push(ParameterValue {
                    id: format!("param{}", self.parameter_values.len()+1),
                    expression: "1.0*m+2.0*mm".into(),
                    display_unit: StandardUnit::MeterFoot,
                })
            }
    
            let mut to_delete: Option<usize> = None;
            let mut to_move: Option<(usize, bool)> = None; // Option containing index and true if moving up and false if down
    
            ui.push_id("paramtable", |ui| {
                egui_extras::TableBuilder::new(ui)
                .striped(true)
                .column(egui_extras::Column::auto())
                .column(egui_extras::Column::auto().at_least(60.0).resizable(true))
                .column(egui_extras::Column::auto().at_least(120.0).resizable(true))
                .column(egui_extras::Column::remainder())
                .header(20.0, |mut header| {
                    header.col(|ui| {
                        ui.label(egui::RichText::new("Edit").strong());
                    });
                    header.col(|ui| {
                        ui.label(egui::RichText::new("ID").strong());
                    });
                    header.col(|ui| {
                        ui.label(egui::RichText::new("Expression").strong());
                    });
                    header.col(|ui| {
                        ui.label(egui::RichText::new("Result").strong());
                    });
                })
                .body(|mut body| {
                    let num_parameters = self.parameter_values.len();
                    for (idx, parameter) in self.parameter_values.iter_mut().enumerate() {
    
    
                        body.row(20.0, |mut row| {
                            row.col(|ui| {
                                ui.horizontal(|ui| {
                                    if ui.button("❌").clicked() {
                                        to_delete = Some(idx);
                                    };
                                    if ui.add_enabled(idx != 0, egui::Button::new("⬆")).clicked() {
                                        to_move = Some((idx, true));
                                    }
                                    if ui.add_enabled(idx < num_parameters - 1, egui::Button::new("⬇")).clicked() {
                                        to_move = Some((idx, false));
                                    };
                                });
                            });
                            row.col(|ui| {
                                ui.add(egui::TextEdit::singleline(&mut parameter.id).clip_text(false));
                            });
                            row.col(|ui| {
                                ui.add(egui::TextEdit::singleline(&mut parameter.expression).clip_text(false));
                            });
                            row.col(|ui| {
                                
                                if evalexpr::Context::get_value(&self.evaluator_context, &parameter.id).is_some() {
                                    ui.label("Error: Identifier already used");
                                }
                                else if let Some(msg) = ChuteDesigner::has_id_error(&parameter.id) {
                                    ui.label(msg);
                                }
                                else {
                                    let computed = evalexpr::eval_number_with_context(&parameter.expression, &self.evaluator_context);
                                    if computed.is_ok() {
                                        let value = computed.unwrap_or_default();
                                        ui.label(format!("Result: {:}", (value * 100_000_000.0).round() / 100_000_000.0));
                    
                                        self.evaluator_context.set_value(parameter.id.clone(), evalexpr::Value::Float(value));
                                    } else {
                                        ui.label(format!("Error: {:?}", computed));
                                    }
                                }
                            });
                            
                        });
                    }
                });    
            });
    
    
            if let Some((idx, direction)) = to_move {
                if idx > 0 && direction {
                    // Swap upwards
                    self.parameter_values.swap(idx, idx-1);
                } else if idx < (self.parameter_values.len() - 1) && !direction{
                    self.parameter_values.swap(idx, idx+1);
                }
            }
    
            if let Some(delete_idx) = to_delete {
                self.parameter_values.remove(delete_idx);
            }
    
    
            ui.add(egui::Hyperlink::from_label_and_url("Info about builtin functions", "https://docs.rs/evalexpr/latest/evalexpr/#builtin-functions"));
            
            ui.separator();
    
            ui = &mut columns[1];
            


            ui.heading("Cross-section geometry");

            ui.horizontal(|ui| {
                if ui.button("Add polygonal geometry").clicked() {
                    todo!();
                }
                
                if ui.button("Add circular band").clicked() {
                    self.chute_sections.push(ChuteSection::new_circular());
                }
            });
    
    
            // Parachute section
    
            let mut to_delete: Option<usize> = None;
            let mut to_move: Option<(usize, bool)> = None; // Option containing index and true if moving up and false if down
            let num_parameters = self.chute_sections.len();
            for (idx, chute_section) in self.chute_sections.iter_mut().enumerate() {
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("❌").clicked() {
                        to_delete = Some(idx);
                    };
                    if ui.add_enabled(idx != 0, egui::Button::new("⬆")).clicked() {
                        to_move = Some((idx, true));
                    }
                    if ui.add_enabled(idx < num_parameters - 1, egui::Button::new("⬇")).clicked() {
                        to_move = Some((idx, false));
                    };
                });
                chute_section.ui(ui, frame, use_imperial, &self.evaluator_context, idx as u16);
            }
    
            if let Some((idx, direction)) = to_move {
                if idx > 0 && direction {
                    // Swap upwards
                    self.chute_sections.swap(idx, idx-1);
                } else if idx < (self.parameter_values.len() - 1) && !direction{
                    self.chute_sections.swap(idx, idx+1);
                }
            }
    
            if let Some(delete_idx) = to_delete {
                self.chute_sections.remove(delete_idx);
            }
    
            ui.separator();
    
            self.draw_cross_section(ui, frame);

        });

    }

    pub fn experiment_ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame, use_imperial: bool) {
        // For measurement data
        // E.g. input velocity/density and get CD
        ui.label("Descent rate");
        ui::length_slider(ui, &mut self.diameter, use_imperial, 0.0..=10.0, &si::velocity::meter_per_second, &si::velocity::foot_per_second);
    }

    pub fn get_cross_section(&self) -> Vec<geometry::Points> {
        // get 2D cross section for display purposes
        let mut result = vec![];

        for chute_section in &self.chute_sections {
            // use relatively low resolution at 30 points
            result.push(chute_section.get_cross_section(30));
        }

        result
    }

    pub fn get_3d_data(&self) {
        todo!()
    }
}


impl Default for ChuteDesigner {
    fn default() -> Self {
        let context = evalexpr::context_map! {
            "m" => 1.0,
            "mm" => 0.001,
            "yd" => 0.9144,
            "ft" => 0.3048,
            "in" => 0.0254,
            "rad" => 1.0,
            "pi" => PI,
            "e" => core::f64::consts::E,
            "deg" => PI / 180.0,
            "ln" => Function::new(|arg| Ok(evalexpr::Value::Float(arg.as_float()?.ln())))
        }.unwrap();

        // TODO: make math functions work without prefix

        let input1 = InputValue {description: "".into(), id: "input1".into(), range: 0.0..=10.0, unit: StandardUnit::MeterFoot, value: 0.0};
        let input2 = InputValue {description: "".into(), id: "input2".into(), range: 0.0..=10.0, unit: StandardUnit::MeterFoot, value: 0.0};

        let input3 = InputValue {description: "Parachute Diameter".into(), id: "diameter".into(), range: 0.0..=10.0, unit: StandardUnit::MeterFoot, value: 1.0};
        let input4 = InputValue {description: "height / diameter of parachute".into(), id: "height_ratio".into(), range: 0.0..=1.0, unit: StandardUnit::UnitLess, value: 0.7};
        let input5 = InputValue {description: "vent_diameter / diameter of parachute".into(), id: "vent_ratio".into(), range: 0.0..=1.0, unit: StandardUnit::UnitLess, value: 0.2};


        let param1 = ParameterValue {display_unit: StandardUnit::MeterFoot, id: "param1".into(), expression: "input1*2".into()};
        let param2 = ParameterValue {display_unit: StandardUnit::MeterFoot, id: "param2".into(), expression: "input2*2".into()};
        let param3 = ParameterValue {display_unit: StandardUnit::MeterFoot, id: "param3".into(), expression: "param1+param2".into()};

        let mut section1 = ChuteSection::new_circular();
        match &mut section1.section_type {
            ChuteSectionType::Circular(sec) => {
                sec.expressions[0] = "vent_ratio*diameter".into();
                sec.expressions[1] = "height_ratio*diameter".into();
                sec.expressions[2] = "diameter".into();
                sec.expressions[3] = "0".into();
            },
            _ => {}
        }

        Self { 
            name: "Untitled Parachute".to_owned(),
            gores: 8,
            diameter: 1.0,
            instructions: vec!["Cut out fabric".to_owned()],
            fabric: FabricSelector::new(),
            use_global_seam_allowance: true,
            global_seam_allowance: 0.01,
            test_color: [0.0, 0.0, 0.0],
            input_values: vec![input1, input2, input3, input4, input5],
            parameter_values: vec![param1, param2, param3],
            evaluator_context: context,
            chute_sections: vec![section1],
        }
    }
}


fn example_write_dxf() -> Result<(), Box<dyn std::error::Error>> {
    // Define the vertices of the triangle
    let vertices = [(0.0, 0.0), (1.0, 0.0), (0.5, 1.0)];

    // Create a new DXF drawing
    let mut drawing = Drawing::new();

    // Create a polyline entity for the triangle
    let mut polyline = Polyline::default();
    polyline.set_is_closed(true); // Closed polyline for a triangle

    // Add vertices to the polyline
    for &(x, y) in &vertices {
        let vertex = Vertex::new(dxf::Point::new(x, y, 0.0));
        polyline.add_vertex(&mut drawing, vertex);
    }

    // Add the polyline to the drawing
    drawing.add_entity(Entity::new(EntityType::Polyline(polyline)));

    // Save the drawing to a DXF file
    drawing.save_file("triangle.dxf")?;

    println!("Triangle saved to triangle.dxf");

    Ok(())
}


// An array of 2D points representing a line in a pattern piece. Seam allowance is constant throughout a segment
// All dimensions given in mm
#[derive(Clone, Debug)]
struct Segment {
    points: Vec<na::Point2<f64>>,
    seam_allowance: f64,
}

impl Segment {
    fn new() -> Self {
        Self { points: vec![], seam_allowance: 0.0 }
    }

    fn new_with_allowance(seam_allowance: f64) -> Self {
        Self { points: vec![], seam_allowance: seam_allowance }
    }

    fn set_seam_allowance(&mut self, seam_allowance: f64) {
        self.seam_allowance = seam_allowance;
    }

    fn add_point(&mut self, point: na::Point2<f64>) {
        self.points.push(point);
    }

    fn add_point_xy(&mut self, x: f64, y: f64) {
        self.points.push(na::Point2::new(x, y));
    }

    fn mirror_x(&self) -> Self {
        // returns a copy mirrored around the X axis
        let mut new = self.clone();
        for point in new.points.iter_mut() {
            point.x = -point.x;
        }
        new
    }

    fn reverse(&self) -> Self {
        let mut new = self.clone();
        new.points.reverse();
        new
    }

    fn scale(&mut self, scale_x: f64, scale_y: f64) {
        // Scale around origin
        for point in self.points.iter_mut() {
            point.x = point.x * scale_x;
            point.y = point.y * scale_y;
        }
    }

    fn get_point(&self, idx: usize) -> na::Point2<f64> {
        self.points[idx].clone()
    }

    fn get_first_point(&self) -> na::Point2<f64> {
        self.points.first().unwrap().clone()
    }

    fn get_last_point(&self) -> na::Point2<f64> {
        self.points.last().unwrap().clone()
    }
}

struct PatternPiece {
    segments: Vec<Segment>,
    points: Vec<na::Point2<f64>>, // Points not including seams
    computed_points: Vec<na::Point2<f64>>, // Final points including seams
    fabric_area: f64, // Area in mm2, includes seam allowances
    chute_area: f64, // Area in mm2, not including seam allowances
    name: String,
}

// Generic arbitrary 2D pattern piece
// Segments are used to allow for different seam allowances
// All the segments are connected together
// Segments NEED to be defined in counterclockwise direction
// Points connecting pieces can be duplicated, but not necessary
// Edge joining two segments is given seam allowance of previous segment
impl PatternPiece {
    fn new() -> PatternPiece {
        Self { segments: vec![], points: vec![], computed_points: vec![], fabric_area: 0.0, chute_area: 0.0, name: "pattern".into() }
    }

    fn add_segment(&mut self, seg: Segment) {
        self.segments.push(seg);
    }

    fn compute(&mut self) {
        // Compute seam allowances etc
        self.computed_points = vec![];
        self.points = vec![];
        for seg in self.segments.iter() {
            self.points.append(&mut seg.points.clone());
            self.computed_points.append(&mut seg.points.clone()); // todo: Add seam computation here
        }
    }

    fn save_dxf(&mut self, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.compute();

        // Create a new DXF drawing
        let mut drawing = Drawing::new();

        // Create a polyline entity for the triangle
        let mut polyline = Polyline::default();
        polyline.set_is_closed(true); // Closed polyline for a triangle

        // Add vertices to the polyline
        for point in self.computed_points.iter() {
            let vertex = Vertex::new(dxf::Point::new(point.x, point.y, 0.0));
            polyline.add_vertex(&mut drawing, vertex);
        }

        // Add the polyline to the drawing
        drawing.add_entity(Entity::new(EntityType::Polyline(polyline)));

        // Save the drawing to a DXF file
        drawing.save_file(filename)?;

        println!("Triangle saved to {}", filename);

        Ok(())

    }

    fn get_area(&self, including_seams: bool) -> f64 {
        let mut points = if including_seams { self.computed_points.clone() } else { self.points.clone() };
        points.push(points.first().unwrap().clone()); // Wrap around

        let mut sum = 0.0;
        // https://en.wikipedia.org/wiki/Shoelace_formula
        for point_pair in points.windows(2) {
            println!("{:?}", point_pair);
            sum += point_pair[0].x * point_pair[1].y - point_pair[0].y * point_pair[1].x;
        }
        sum/2.0
    }
}

struct PatternPieceCollection {
    pieces: Vec<(PatternPiece, u32)>, // Pattern piece and number of each
}

impl PatternPieceCollection {
    fn new() -> Self {
        Self { pieces: vec![] }
    }

    fn get_area(&self, including_seams: bool) -> f64 {
        let mut total_area = 0.0;

        for (piece, count) in self.pieces.iter() {
            total_area += piece.get_area(including_seams) * (*count as f64);
        }

        total_area
    }
}

// Standard gore geometry with four sides and straight top/bottom section
// If coords_left is None, the coords are mirrored on the left side
// Note: Coordinates go from bottom to top for BOTH left and right sections
struct Gore {
    coords_right: Segment,
    coords_left: Segment,
    seam_allowances: (f64, f64, f64, f64), // Seams. Right, top, left, bottom
}

impl Gore {
    fn new(coords_right: Segment, coords_left: Segment, seam_allowances: (f64, f64, f64, f64)) -> Self {
        // Coords are defined from bottom to top
        Self { coords_right: coords_right, coords_left: coords_left, seam_allowances: seam_allowances}
    }

    fn new_symmetric(coords_right: Segment, seam_allowances: (f64, f64, f64, f64)) -> Self {
        Gore::new(coords_right.clone(), coords_right.clone().mirror_x(), seam_allowances)
    }

    fn get_pattern_piece(&self, corner_cutout: bool) -> PatternPiece {
        let mut piece = PatternPiece::new();
        // Add right segment, excluding top point

        let mut segment_right = self.coords_right.clone();
        segment_right.set_seam_allowance(self.seam_allowances.0);

        let mut segment_left = self.coords_left.reverse();
        segment_left.set_seam_allowance(self.seam_allowances.2);
        
        let mut segment_top = Segment::new_with_allowance(self.seam_allowances.1);
        segment_top.add_point(segment_right.get_last_point());
        segment_top.add_point(segment_left.get_first_point());

        let mut segment_bottom = Segment::new_with_allowance(self.seam_allowances.3);
        segment_bottom.add_point(segment_left.get_last_point());
        segment_bottom.add_point(segment_right.get_first_point());
        
        piece.add_segment(segment_right);
        piece.add_segment(segment_top);
        piece.add_segment(segment_left);
        piece.add_segment(segment_bottom);

        piece
    }
}


// Trait for different parachute types
trait Parachute {
    fn get_pieces(&self) -> PatternPieceCollection;
    fn get_fabric_area(&self) -> f64;
    fn get_parachute_area(&self) -> f64;
    fn get_projected_area(&self) -> f64;
    fn get_cd(&self) -> f64; // Drag coefficient, based on parachute_area
    fn get_line_length(&self) -> f64;
}

struct ParachuteProject {
    chute: Box<dyn Parachute>
}


// Suspension line stuff
struct SuspensionLine {
    rating_newtons: f64,
    linear_density_g_m: f64,
    name: String,
}

#[derive(Default, Clone, PartialEq)]
struct Fabric {
    area_density_gsm: f64,
    name: String
}

impl Fabric {
    fn new(gsm: f64, name: &str) -> Self {
        Self { area_density_gsm: gsm, name: name.to_owned() }
    }

    fn get_name_weight(&self, imperial: bool) -> String {
        if imperial {
            format!("{} ({:.1} oz)", self.name, self.area_density_gsm / 33.906)
        } else {
            format!("{} ({:.0} gsm)", self.name, self.area_density_gsm)
        }
    }

}

#[derive(PartialEq, Clone)]
struct FabricSelector {
    modified: bool,
    selected_fabric: Fabric,
    fabric_options: Vec<Fabric>,
}

impl FabricSelector {
    fn new() -> Self {

        let mut options = vec![];

        options.push(Fabric::new(38.0, "Ripstop nylon"));
        options.push(Fabric::new(48.0, "Ripstop nylon"));
        options.push(Fabric::new(67.0, "Ripstop nylon"));

        let default_fabric = Fabric::new(38.0, "Ripstop nylon");

        Self { modified: false, fabric_options: options, selected_fabric: default_fabric }
    }


    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame, use_imperial: bool, id: u16) {
        ui.label("Select fabric:");
        egui::ComboBox::from_id_source(id)
            .width(200.0)
            .selected_text(self.selected_fabric.get_name_weight(use_imperial))
            .show_ui(ui, |ui| {
                for (_idx, option) in self.fabric_options.iter().enumerate() {
                    ui.selectable_value(&mut self.selected_fabric, option.clone(), option.get_name_weight(use_imperial));
                }
            }
        );
    }
}

impl Default for FabricSelector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::chute::parachute::{Segment, PatternPiece};
    extern crate nalgebra as na;

    #[test]
    fn test_save_dxf() {
        let mut seg = Segment::new();
        seg.add_point_xy(0.0, 0.0);
        seg.add_point_xy(1.0, 0.0);
        seg.add_point_xy(0.5, 2.0);

        let mut pat = PatternPiece::new();
        pat.add_segment(seg);
        pat.compute();
        pat.save_dxf("tall_triangle.dxf").unwrap();
    }

    #[test]
    fn test_mirror_scale() {
        let mut seg = Segment::new();
        seg.add_point_xy(0.0, 0.0);
        seg.add_point_xy(1.0, 2.0);

        println!("{:?}", seg);
        seg.mirror_x();
        assert_eq!(seg.points[1], na::Point2::new(-1.0, 2.0));
        println!("{:?}", seg);
        seg.scale(2.0, 3.0);
        assert_eq!(seg.points[1], na::Point2::new(-2.0, 6.0));
    }

    #[test]
    fn test_area() {
        let mut seg = Segment::new();
        // Funky triangle, should give area of 2
        seg.add_point_xy(0.0, 0.0);
        seg.add_point_xy(1.0, 0.0);
        seg.add_point_xy(-0.5, 4.0);

        let mut pat = PatternPiece::new();
        pat.add_segment(seg);
        pat.compute();

        assert_eq!(pat.get_area(false), 1.0 * 4.0 * 0.5)
    }
}