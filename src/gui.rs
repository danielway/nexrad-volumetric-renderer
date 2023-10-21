use crate::param::ClusteringMode::{DBSCAN, KNN};
use crate::param::InteractionMode::{ManualOrbit, Orbit};
use crate::param::Parameters;
use crate::param::PointWeightMode::{Density, Hybrid, ReturnStrength};
use crate::CONTROL_PANEL_WIDTH;
use chrono::{NaiveDate, NaiveTime};
use std::str::FromStr;
use three_d::egui::Context;

pub fn render_gui(gui_context: &Context, params: &mut Parameters) {
    use three_d::egui::*;
    SidePanel::left("side_panel")
        .exact_width(CONTROL_PANEL_WIDTH)
        .resizable(false)
        .show(gui_context, |ui| {
            ui.add_space(10.0);

            ui.heading("Control Panel");

            ui.add_space(10.0);

            ui.label("Site");
            ui.text_edit_singleline(&mut params.site);

            ui.label("Date");
            let mut date_string = params.date.to_string();
            let date_input = ui.text_edit_singleline(&mut date_string);
            if date_input.changed() {
                match NaiveDate::from_str(&date_string) {
                    Ok(date) => params.date = date,
                    Err(_) => {}
                }
            }

            ui.label("Time");
            let mut time_string = params.time.to_string();
            let time_input = ui.text_edit_singleline(&mut time_string);
            if time_input.changed() {
                match NaiveTime::from_str(&time_string) {
                    Ok(time) => params.time = time,
                    Err(_) => {}
                }
            }

            ui.add_space(10.0);

            ui.label("Interaction Mode");
            ui.radio_value(&mut params.interaction_mode, Orbit, "Orbit");
            ui.radio_value(&mut params.interaction_mode, ManualOrbit, "Manual");

            ui.add_space(10.0);

            ui.label("Data Sampling");
            let mut data_sampling_string = params.data_sampling.to_string();
            let data_sampling_input = ui.text_edit_singleline(&mut data_sampling_string);
            if data_sampling_input.changed() {
                match u16::from_str(&data_sampling_string) {
                    Ok(data_sampling) => params.data_sampling = data_sampling,
                    Err(_) => {}
                }
            }

            ui.add_space(10.0);

            ui.label("Point Weight Mode");
            ui.radio_value(&mut params.point_weight_mode, ReturnStrength, "Return");
            ui.radio_value(&mut params.point_weight_mode, Density, "Density");
            ui.radio_value(&mut params.point_weight_mode, Hybrid, "Hybrid");

            ui.add_space(10.0);

            ui.label("Clustering Mode");
            ui.radio_value(&mut params.clustering_mode, KNN, "KNN");
            ui.radio_value(&mut params.clustering_mode, DBSCAN, "DBSCAN");

            ui.add_space(10.0);

            ui.label("Clustering Threshold");
            let mut clustering_threshold_string = params.clustering_threshold.to_string();
            let clustering_threshold_input =
                ui.text_edit_singleline(&mut clustering_threshold_string);
            if clustering_threshold_input.changed() {
                match f32::from_str(&clustering_threshold_string) {
                    Ok(clustering_threshold) => params.clustering_threshold = clustering_threshold,
                    Err(_) => {}
                }
            }
        });
}
