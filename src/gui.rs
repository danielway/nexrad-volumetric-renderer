use crate::param::ClusteringMode::{DBSCAN, KNN};
use crate::param::InteractionMode::{ManualOrbit, Orbit};
use crate::param::Parameters;
use crate::param::PointColorMode::{Density, Hybrid, Raw};
use crate::CONTROL_PANEL_WIDTH;
use chrono::{NaiveDate, NaiveTime};
use std::str::FromStr;
use three_d::egui::Context;

pub struct GuiState {
    pub date_string: String,
    pub time_string: String,
    pub data_sampling_string: String,
    pub clustering_threshold_string: String,
}

pub fn render_gui(gui_context: &Context, state: &mut GuiState, params: &mut Parameters) -> (bool, bool) {
    use three_d::egui::*;
    
    let mut refetch_data = false;
    let mut reprocess_data = false;
    
    SidePanel::left("side_panel")
        .exact_width(CONTROL_PANEL_WIDTH)
        .resizable(false)
        .show(gui_context, |ui| {
            ui.add_space(10.0);

            ui.heading("Control Panel");

            ui.add_space(10.0);

            ui.columns(2, |columns| {
                columns[0].label("Site");
                
                let site_input = columns[1].text_edit_singleline(&mut params.site);
                if site_input.changed() {
                    refetch_data = true;
                }
            });

            ui.columns(2, |columns| {
                columns[0].label("Date");

                let date_input = columns[1].text_edit_singleline(&mut state.date_string);
                if date_input.changed() {
                    match NaiveDate::from_str(&state.date_string) {
                        Ok(date) => {
                            params.date = date;
                            refetch_data = true;
                        }
                        Err(_) => {}
                    }
                }
            });

            ui.columns(2, |columns| {
                columns[0].label("Time");

                let time_input = columns[1].text_edit_singleline(&mut state.time_string);
                if time_input.changed() {
                    match NaiveTime::from_str(&state.time_string) {
                        Ok(time) => {
                            params.time = time;
                            refetch_data = true;
                        },
                        Err(_) => {}
                    }
                }
            });

            ui.add_space(10.0);

            ui.label("Interaction Mode");
            ui.radio_value(&mut params.interaction_mode, Orbit, "Orbit");
            ui.radio_value(&mut params.interaction_mode, ManualOrbit, "Manual");

            ui.add_space(10.0);

            ui.separator();

            ui.add_space(10.0);

            ui.label("Data Sampling");
            
            let data_sampling_input = ui.text_edit_singleline(&mut state.data_sampling_string);
            if data_sampling_input.changed() {
                match u16::from_str(&state.data_sampling_string) {
                    Ok(data_sampling) => {
                        params.data_sampling = data_sampling;
                        reprocess_data = true;
                    },
                    Err(_) => {}
                }
            }

            ui.add_space(10.0);

            ui.label("Point Color Mode");
            
            let raw_input = ui.radio_value(&mut params.point_color_mode, Raw, "Raw");
            if raw_input.changed() {
                reprocess_data = true;
            }
            
            let density_input = ui.radio_value(&mut params.point_color_mode, Density, "Density");
            if density_input.changed() {
                reprocess_data = true;
            }
            
            let hybrid_input = ui.radio_value(&mut params.point_color_mode, Hybrid, "Hybrid");
            if hybrid_input.changed() {
                reprocess_data = true;
            }

            ui.add_space(10.0);

            ui.separator();

            ui.add_space(10.0);

            ui.label("Clustering Mode");
            
            let knn_input = ui.radio_value(&mut params.clustering_mode, KNN, "KNN");
            if knn_input.changed() {
                reprocess_data = true;
            }
            
            let dbscan_input = ui.radio_value(&mut params.clustering_mode, DBSCAN, "DBSCAN");
            if dbscan_input.changed() {
                reprocess_data = true;
            }

            ui.add_space(10.0);

            ui.label("Clustering Threshold");
            let clustering_threshold_input =
                ui.text_edit_singleline(&mut state.clustering_threshold_string);
            if clustering_threshold_input.changed() {
                match f32::from_str(&state.clustering_threshold_string) {
                    Ok(clustering_threshold) => {
                        params.clustering_threshold = clustering_threshold;
                        reprocess_data = true;
                    }
                    Err(_) => {}
                }
            }
        });
    
    reprocess_data |= refetch_data;

    (refetch_data, reprocess_data)
}
