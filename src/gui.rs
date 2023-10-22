use crate::param::ClusteringMode::{DBSCAN, KNN};
use crate::param::InteractionMode::{ManualOrbit, Orbit};
use crate::param::Parameters;
use crate::param::PointColorMode::{Density, Hybrid, Raw};
use crate::processing::do_fetch_and_process;
use crate::state::State;
use crate::CONTROL_PANEL_WIDTH;
use chrono::{NaiveDate, NaiveTime};
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use three_d::egui::Context;

pub struct GuiState {
    pub date_string: String,
    pub time_string: String,
    pub data_sampling_string: String,
    pub clustering_threshold_string: String,
}

pub fn render_gui(
    gui_context: &Context,
    state: &Arc<Mutex<State>>,
    gui_state: &mut GuiState,
    params: &mut Parameters,
) -> bool {
    use three_d::egui::*;

    let mut should_rerender = false;

    let processing = {
        let state = state.lock().unwrap();
        state.processing
    };

    SidePanel::left("side_panel")
        .exact_width(CONTROL_PANEL_WIDTH)
        .resizable(false)
        .show(gui_context, |ui| {
            ui.add_space(10.0);

            ui.heading("Control Panel");

            if processing {
                ui.colored_label(Color32::from_rgb(255, 0, 0), "Processing data...");
            }

            ui.add_space(10.0);

            ui.columns(2, |columns| {
                columns[0].label("Site");
                columns[1].text_edit_singleline(&mut params.site);
            });

            ui.columns(2, |columns| {
                columns[0].label("Date");

                let date_input = columns[1].text_edit_singleline(&mut gui_state.date_string);
                if date_input.changed() {
                    match NaiveDate::from_str(&gui_state.date_string) {
                        Ok(date) => params.date = date,
                        Err(_) => {}
                    }
                }
            });

            ui.columns(2, |columns| {
                columns[0].label("Time");

                let time_input = columns[1].text_edit_singleline(&mut gui_state.time_string);
                if time_input.changed() {
                    match NaiveTime::from_str(&gui_state.time_string) {
                        Ok(time) => params.time = time,
                        Err(_) => {}
                    }
                }
            });

            let fetch_button = ui.button("Fetch");
            if fetch_button.clicked() {
                do_fetch_and_process(params.site.clone(), params.date, params.time, state.clone());

                should_rerender = true;
            }

            ui.add_space(10.0);

            ui.label("Interaction Mode");
            ui.radio_value(&mut params.interaction_mode, Orbit, "Orbit");
            ui.radio_value(&mut params.interaction_mode, ManualOrbit, "Manual");

            ui.add_space(10.0);

            ui.separator();

            ui.add_space(10.0);

            ui.label("Data Sampling");

            let data_sampling_input = ui.text_edit_singleline(&mut gui_state.data_sampling_string);
            if data_sampling_input.changed() {
                match u16::from_str(&gui_state.data_sampling_string) {
                    Ok(data_sampling) => params.data_sampling = data_sampling,
                    Err(_) => {}
                }
            }

            ui.add_space(10.0);

            ui.label("Point Color Mode");
            ui.radio_value(&mut params.point_color_mode, Raw, "Raw");
            ui.radio_value(&mut params.point_color_mode, Density, "Density");
            ui.radio_value(&mut params.point_color_mode, Hybrid, "Hybrid");

            ui.add_space(10.0);

            ui.separator();

            ui.add_space(10.0);

            ui.label("Clustering Mode");
            ui.radio_value(&mut params.clustering_mode, KNN, "KNN");
            ui.radio_value(&mut params.clustering_mode, DBSCAN, "DBSCAN");

            ui.add_space(10.0);

            ui.label("Clustering Threshold");
            let clustering_threshold_input =
                ui.text_edit_singleline(&mut gui_state.clustering_threshold_string);
            if clustering_threshold_input.changed() {
                match f32::from_str(&gui_state.clustering_threshold_string) {
                    Ok(clustering_threshold) => params.clustering_threshold = clustering_threshold,
                    Err(_) => {}
                }
            }
        });

    should_rerender
}
