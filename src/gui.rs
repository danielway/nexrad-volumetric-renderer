use crate::param::ClusteringMode::{DBSCAN, KNN};
use crate::param::InteractionMode::{ManualOrbit, Orbit};
use crate::param::PointColorMode::{Density, Hybrid, Raw};
use crate::param::{ClusteringMode, DataParams, VisParams};
use crate::state::State;
use crate::CONTROL_PANEL_WIDTH;
use chrono::{NaiveDate, NaiveTime};
use std::str::FromStr;
use three_d::egui::{Align, Color32, Layout, SidePanel, Ui};
use three_d::{Context, FrameInput, GUI};

pub struct Gui {
    gui: Option<GUI>,
    site_string: String,
    date_string: String,
    time_string: String,
    sampling_string: String,
    clustering_mode: ClusteringMode,
    clustering_t_string: String,
}

impl Gui {
    pub fn new(context: &Context, parameters: &DataParams) -> Self {
        Gui {
            gui: Some(GUI::new(context)),
            site_string: parameters.site.to_string(),
            date_string: parameters.date.to_string(),
            time_string: parameters.time.to_string(),
            sampling_string: parameters.sampling.to_string(),
            clustering_mode: parameters.clustering_mode,
            clustering_t_string: parameters.clustering_threshold.to_string(),
        }
    }

    pub fn update(
        &mut self,
        frame_input: &mut FrameInput,
        state: &State,
        vis_params: &VisParams,
        data_params: &DataParams,
    ) -> (Option<VisParams>, Option<DataParams>) {
        let mut new_vis_params: Option<VisParams> = None;
        let mut new_data_params: Option<DataParams> = None;

        // todo: not idiomatic
        let mut gui = self.gui.take().unwrap();
        gui.update(
            &mut frame_input.events,
            frame_input.accumulated_time,
            frame_input.viewport,
            frame_input.device_pixel_ratio,
            |gui_context| {
                SidePanel::left("side_panel")
                    .exact_width(CONTROL_PANEL_WIDTH)
                    .resizable(false)
                    .show(gui_context, |ui| {
                        ui.add_space(10.0);

                        new_vis_params = self.update_vis_params(ui, vis_params);
                        self.update_current_params(ui, data_params);
                        new_data_params = self.update_data_params(ui);
                        self.update_status(ui, state);
                    });
            },
        );
        self.gui = Some(gui);

        (new_vis_params, new_data_params)
    }

    fn update_vis_params(&mut self, ui: &mut Ui, vis_params: &VisParams) -> Option<VisParams> {
        let mut new_vis_params = vis_params.clone();

        ui.heading("Visualization");

        ui.add_space(10.0);

        ui.label("Interaction Mode");
        ui.radio_value(&mut new_vis_params.interaction_mode, Orbit, "Orbit");
        ui.radio_value(&mut new_vis_params.interaction_mode, ManualOrbit, "Manual");

        ui.add_space(10.0);

        ui.label("Point Color Mode");
        ui.radio_value(&mut new_vis_params.point_color_mode, Raw, "Raw");
        ui.radio_value(&mut new_vis_params.point_color_mode, Density, "Density");
        ui.radio_value(&mut new_vis_params.point_color_mode, Hybrid, "Hybrid");

        ui.add_space(10.0);

        if &new_vis_params == vis_params {
            None
        } else {
            Some(new_vis_params)
        }
    }

    fn update_current_params(&self, ui: &mut Ui, data_params: &DataParams) {
        ui.heading("Current Parameters");

        ui.add_space(10.0);

        ui.columns(2, |columns| {
            columns[0].label("Site");
            columns[1].colored_label(Color32::from_rgb(255, 255, 255), &data_params.site);

            columns[0].label("Date");
            columns[1].colored_label(
                Color32::from_rgb(255, 255, 255),
                data_params.date.to_string(),
            );

            columns[0].label("Time");
            columns[1].colored_label(
                Color32::from_rgb(255, 255, 255),
                data_params.time.to_string(),
            );

            columns[0].label("Sampling");
            columns[1].colored_label(
                Color32::from_rgb(255, 255, 255),
                data_params.sampling.to_string(),
            );

            columns[0].label("Cluster Mode");
            columns[1].colored_label(
                Color32::from_rgb(255, 255, 255),
                format!("{:?}", data_params.clustering_mode),
            );

            columns[0].label("Cl. Threshold");
            columns[1].colored_label(
                Color32::from_rgb(255, 255, 255),
                data_params.clustering_threshold.to_string(),
            );
        });

        ui.add_space(10.0);
    }

    fn update_data_params(&mut self, ui: &mut Ui) -> Option<DataParams> {
        ui.heading("Update Parameters");

        ui.add_space(10.0);

        ui.columns(2, |columns| {
            columns[0].label("Site");
            columns[1].text_edit_singleline(&mut self.site_string);
        });

        ui.columns(2, |columns| {
            columns[0].label("Date");
            columns[1].text_edit_singleline(&mut self.date_string);
        });

        ui.columns(2, |columns| {
            columns[0].label("Time");
            columns[1].text_edit_singleline(&mut self.time_string);
        });

        ui.columns(2, |columns| {
            columns[0].label("Sampling");
            columns[1].text_edit_singleline(&mut self.sampling_string);
        });

        ui.add_space(10.0);

        ui.label("Cluster Mode");
        ui.radio_value(&mut self.clustering_mode, KNN, "KNN");
        ui.radio_value(&mut self.clustering_mode, DBSCAN, "DBSCAN");

        ui.add_space(10.0);

        ui.label("Cluster Threshold");
        ui.text_edit_singleline(&mut self.clustering_t_string);

        ui.add_space(10.0);

        let apply_button = ui.button("Apply");

        ui.add_space(10.0);

        if apply_button.clicked() {
            return Some(DataParams {
                site: self.site_string.clone(),
                date: NaiveDate::from_str(&self.date_string).unwrap(),
                time: NaiveTime::from_str(&self.time_string).unwrap(),
                sampling: u16::from_str(&self.sampling_string).unwrap(),
                clustering_mode: self.clustering_mode,
                clustering_threshold: f32::from_str(&self.clustering_t_string).unwrap(),
            });
        }

        None
    }

    fn update_status(&self, ui: &mut Ui, state: &State) {
        ui.with_layout(Layout::bottom_up(Align::Center), |ui| {
            ui.add_space(10.0);

            if let Some(ref stats) = state.statistics {
                ui.label(format!(
                    "Load {}, Decompress {}, Decode {}, \
                    Pointing {}, Sampling {}, Coloring {}",
                    stats.load_ms,
                    stats.decompress_ms,
                    stats.decode_ms,
                    stats.pointing_ms,
                    stats.sampling_ms,
                    stats.coloring_ms,
                ));
            }

            if state.processing {
                ui.colored_label(Color32::from_rgb(255, 0, 0), "Processing data...");
            }
        });
    }

    pub fn render(&self, frame_input: &FrameInput) {
        frame_input
            .screen()
            .write(|| self.gui.as_ref().unwrap().render());
    }
}
