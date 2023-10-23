use crate::param::ClusteringMode::{DBSCAN, KNN};
use crate::param::InteractionMode::{ManualOrbit, Orbit};
use crate::param::PointColorMode::{Density, Hybrid, Raw};
use crate::param::{DataParameters, RenderParameters};
use crate::state::State;
use crate::CONTROL_PANEL_WIDTH;
use chrono::{NaiveDate, NaiveTime};
use std::str::FromStr;
use three_d::egui::{Color32, SidePanel};
use three_d::{Context, FrameInput, GUI};

pub struct Gui {
    gui: GUI,
    site_string: String,
    date_string: String,
    time_string: String,
    data_sampling_string: String,
    clustering_threshold_string: String,
}

impl Gui {
    pub fn new(context: &Context, parameters: &DataParameters) -> Self {
        Gui {
            gui: GUI::new(context),
            site_string: parameters.site.to_string(),
            date_string: parameters.date.to_string(),
            time_string: parameters.time.to_string(),
            data_sampling_string: parameters.data_sampling.to_string(),
            clustering_threshold_string: parameters.clustering_threshold.to_string(),
        }
    }

    pub fn update(
        &mut self,
        frame_input: &mut FrameInput,
        state: &State,
        render_parameters: &RenderParameters,
        data_parameters: &DataParameters,
    ) -> (Option<RenderParameters>, Option<DataParameters>) {
        let mut new_render_parameters = render_parameters.clone();
        let mut new_data_parameters = data_parameters.clone();

        self.gui.update(
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

                        ui.heading("Control Panel");

                        if state.processing {
                            ui.colored_label(Color32::from_rgb(255, 0, 0), "Processing data...");
                        }

                        // todo: render current parameters

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

                        let fetch_button = ui.button("Fetch");
                        if fetch_button.clicked() {
                            new_data_parameters.site = self.site_string.to_string();
                            new_data_parameters.date =
                                NaiveDate::from_str(&self.date_string).unwrap();
                            new_data_parameters.time =
                                NaiveTime::from_str(&self.time_string).unwrap();
                        }

                        ui.add_space(10.0);

                        ui.label("Interaction Mode");
                        ui.radio_value(&mut new_render_parameters.interaction_mode, Orbit, "Orbit");
                        ui.radio_value(
                            &mut new_render_parameters.interaction_mode,
                            ManualOrbit,
                            "Manual",
                        );

                        ui.add_space(10.0);

                        ui.separator();

                        ui.add_space(10.0);

                        ui.label("Data Sampling");

                        let data_sampling_input =
                            ui.text_edit_singleline(&mut self.data_sampling_string);
                        if data_sampling_input.changed() {
                            match u16::from_str(&self.data_sampling_string) {
                                Ok(data_sampling) => {
                                    new_data_parameters.data_sampling = data_sampling
                                }
                                Err(_) => {}
                            }
                        }

                        ui.add_space(10.0);

                        ui.label("Point Color Mode");
                        ui.radio_value(&mut new_render_parameters.point_color_mode, Raw, "Raw");
                        ui.radio_value(
                            &mut new_render_parameters.point_color_mode,
                            Density,
                            "Density",
                        );
                        ui.radio_value(
                            &mut new_render_parameters.point_color_mode,
                            Hybrid,
                            "Hybrid",
                        );

                        ui.add_space(10.0);

                        ui.separator();

                        ui.add_space(10.0);

                        ui.label("Clustering Mode");
                        ui.radio_value(&mut new_data_parameters.clustering_mode, KNN, "KNN");
                        ui.radio_value(&mut new_data_parameters.clustering_mode, DBSCAN, "DBSCAN");

                        ui.add_space(10.0);

                        ui.label("Clustering Threshold");
                        let clustering_threshold_input =
                            ui.text_edit_singleline(&mut self.clustering_threshold_string);
                        if clustering_threshold_input.changed() {
                            match f32::from_str(&self.clustering_threshold_string) {
                                Ok(clustering_threshold) => {
                                    new_data_parameters.clustering_threshold = clustering_threshold
                                }
                                Err(_) => {}
                            }
                        }
                    });
            },
        );

        (
            if &new_render_parameters == render_parameters {
                None
            } else {
                Some(new_render_parameters)
            },
            if &new_data_parameters == data_parameters {
                None
            } else {
                Some(new_data_parameters)
            },
        )
    }

    pub fn render(&self, frame_input: &FrameInput) {
        frame_input.screen().write(|| self.gui.render());
    }
}
