use crate::data::{get_data, get_points};
use crate::gui::render_gui;
use crate::object::{get_earth_object, get_point_cloud_object, get_radar_indicator_object};
use crate::param::{ClusteringMode, InteractionMode, Parameters, PointWeightMode};
use chrono::{NaiveDate, NaiveTime};
use three_d::{ClearState, FrameOutput, Viewport, Window, WindowSettings, GUI};

use crate::result::Result;
use crate::scene::{get_camera_and_control, get_sun_light, do_auto_orbit};

mod data;
mod gui;
mod object;
mod param;
mod result;
mod scene;

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

    let decoded = get_data(site, date, time).await?;
    let points = get_points(&decoded, 0.5);

    // Sample dataset to speed processing
    let sampled_points = points.iter().step_by(100).collect::<Vec<_>>();
    println!("Scan contains {} points.", sampled_points.len());

    let point_cloud_gm = get_point_cloud_object(&context, sampled_points);

    let (mut camera, mut control) = get_camera_and_control(&window);
    let mut gui = GUI::new(&context);

    let mut angle_deg = 0.0;
    let mut parameters = Parameters {
        site: site.to_string(),
        date: *date,
        time: *time,
        interaction_mode: InteractionMode::Orbit,
        data_sampling: 100,
        clustering_mode: ClusteringMode::DBSCAN,
        point_weight_mode: PointWeightMode::ReturnStrength,
        clustering_threshold: 10.0,
    };

    window.render_loop(move |mut frame_input| {
        let scaled_width = CONTROL_PANEL_WIDTH * frame_input.device_pixel_ratio;
        camera.set_viewport(Viewport {
            x: scaled_width as i32,
            y: 0,
            width: frame_input.viewport.width - scaled_width as u32,
            height: frame_input.viewport.height,
        });

        control.handle_events(&mut camera, &mut frame_input.events);

        if parameters.interaction_mode == InteractionMode::Orbit {
            do_auto_orbit(&mut angle_deg, &mut camera);
        }

        let objects = point_cloud_gm
            .into_iter()
            .chain(&earth)
            .chain(&radar_indicator);

        gui.update(
            &mut frame_input.events,
            frame_input.accumulated_time,
            frame_input.viewport,
            frame_input.device_pixel_ratio,
            |gui_context| {
                render_gui(gui_context, &mut parameters);
            },
        );

        frame_input
            .screen()
            .clear(ClearState::color_and_depth(0.0, 0.0, 0.0, 1.0, 1.0))
            .render(&camera, objects, &[&sun]);

        frame_input.screen().write(|| gui.render());

        FrameOutput::default()
    });

    Ok(())
}
