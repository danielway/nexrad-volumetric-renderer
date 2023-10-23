use crate::gui::Gui;
use crate::object::{get_earth_object, get_point_cloud_object, get_radar_indicator_object};
use crate::param::{ClusteringMode, DataParams, InteractionMode, PointColorMode, VisParams};
use crate::processing::do_fetch_and_process;
use chrono::{NaiveDate, NaiveTime};
use dbscan::Classification;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use three_d::{
    ClearState, ColorMaterial, FrameOutput, Gm, InstancedMesh, Vector3, Viewport, Window,
    WindowSettings,
};

use crate::result::Result;
use crate::scene::{do_auto_orbit, get_camera_and_control, get_sun_light};
use crate::state::State;

mod data;
mod gui;
mod object;
mod param;
mod processing;
mod result;
mod scene;
mod state;

const TARGET_SITE: &str = "KDMX";

#[tokio::main]
async fn main() {
    let target_date = NaiveDate::from_ymd_opt(2022, 3, 5).unwrap();
    let target_time = NaiveTime::from_hms_opt(23, 30, 0).unwrap();

    execute(TARGET_SITE, &target_date, &target_time)
        .await
        .unwrap();
}

const RENDER_RATIO_TO_M: f32 = 0.00001; // every 1.0 in the render == 1.0/RENDER_RATIO_TO_M meters

const CONTROL_PANEL_WIDTH: f32 = 200.0;

type ColoredPoint = (Vector3<f32>, (u8, u8, u8));

async fn execute(site: &str, date: &NaiveDate, time: &NaiveTime) -> Result<()> {
    let window = Window::new(WindowSettings {
        title: "NEXRAD Volumetric Renderer".to_string(),
        max_size: Some((1280, 720)),
        ..Default::default()
    })?;

    let context = window.gl();

    let earth = get_earth_object(&context);
    let radar_indicator = get_radar_indicator_object(&context);
    let sun = get_sun_light(&context);

    let state = Arc::new(Mutex::new(State {
        processing: false,
        points: None,
    }));

    do_fetch_and_process(site.to_string(), *date, *time, state.clone());

    let mut render_parameters = VisParams {
        interaction_mode: InteractionMode::ManualOrbit,
        point_color_mode: PointColorMode::Raw,
    };

    let mut data_parameters = DataParams {
        site: site.to_string(),
        date: *date,
        time: *time,
        sampling: 100,
        clustering_mode: ClusteringMode::DBSCAN,
        clustering_threshold: 10.0,
    };

    let (mut camera, mut control) = get_camera_and_control(&window);
    let mut gui = Gui::new(&context, &data_parameters);

    let mut angle_deg = 0.0;

    let mut point_cloud: Option<Gm<InstancedMesh, ColorMaterial>> = None;

    window.render_loop(move |mut frame_input| {
        let scaled_width = CONTROL_PANEL_WIDTH * frame_input.device_pixel_ratio;
        camera.set_viewport(Viewport {
            x: scaled_width as i32,
            y: 0,
            width: frame_input.viewport.width - scaled_width as u32,
            height: frame_input.viewport.height,
        });

        control.handle_events(&mut camera, &mut frame_input.events);

        if render_parameters.interaction_mode == InteractionMode::Orbit {
            do_auto_orbit(&mut angle_deg, &mut camera);
        }

        {
            let current_state = state.lock().unwrap();
            let (updated_render_parameters, updated_data_parameters) = gui.update(
                &mut frame_input,
                &current_state,
                &render_parameters,
                &data_parameters,
            );

            if let Some(updated_render_parameters) = updated_render_parameters {
                render_parameters = updated_render_parameters;
            }

            if let Some(updated_data_parameters) = updated_data_parameters {
                data_parameters = updated_data_parameters;
                point_cloud = None;
                do_fetch_and_process(
                    data_parameters.site.clone(),
                    data_parameters.date,
                    data_parameters.time,
                    state.clone(),
                );
            }
        }

        let objects = earth.into_iter().chain(&radar_indicator);

        if point_cloud.is_none() {
            {
                let mut state = state.lock().unwrap();
                if !state.processing && state.points.is_some() {
                    let points = state.points.take().unwrap();
                    point_cloud = Some(get_point_cloud_object(&context, points));
                }
            }
        }

        if let Some(ref point_cloud) = point_cloud {
            frame_input
                .screen()
                .clear(ClearState::color_and_depth(0.0, 0.0, 0.0, 1.0, 1.0))
                .render(&camera, objects.chain(point_cloud), &[&sun]);
        } else {
            frame_input
                .screen()
                .clear(ClearState::color_and_depth(0.0, 0.0, 0.0, 1.0, 1.0))
                .render(&camera, objects, &[&sun]);
        };

        gui.render(&frame_input);
        FrameOutput::default()
    });

    Ok(())
}

// Returns: (clustered points, unclustered points)
fn do_dbscan_clustering(points: Vec<ColoredPoint>) -> (Vec<Vec<ColoredPoint>>, Vec<ColoredPoint>) {
    let mut clusters = HashMap::new();
    let mut unclustered_points = Vec::new();

    let vectorized_points: Vec<Vec<f32>> = points
        .iter()
        .map(|(p, _)| vec![p.x, p.y, p.z])
        .collect::<Vec<_>>();

    let classifications = dbscan::cluster(0.05, 5, &vectorized_points);

    for (colored_point, classification) in points.into_iter().zip(classifications) {
        match classification {
            Classification::Core(cluster) => {
                if !clusters.contains_key(&cluster) {
                    clusters.insert(cluster, Vec::new());
                }

                clusters.get_mut(&cluster).unwrap().push(colored_point);
            }
            _ => unclustered_points.push(colored_point),
        }
    }

    (
        clusters
            .into_iter()
            .map(|(_, points)| points)
            .collect::<Vec<_>>(),
        unclustered_points,
    )
}
