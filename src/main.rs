use chrono::{NaiveDate, NaiveTime};
use kiss3d::light::Light;
use kiss3d::nalgebra::{Point3, Translation3};
use kiss3d::window::Window;
use nexrad::decode::decode_file;
use nexrad::decompress::decompress_file;
use nexrad::download::{download_file, list_files};
use nexrad::file::FileMetadata;
use std::f32::consts::PI;

use crate::result::Result;

mod result;

const TARGET_SITE: &str = "KDMX";

#[tokio::main]
async fn main() {
    let target_date = NaiveDate::from_ymd_opt(2022, 3, 5).unwrap();
    let target_time = NaiveTime::from_hms_opt(23, 30, 0).unwrap();

    execute(TARGET_SITE, &target_date, &target_time)
        .await
        .unwrap();
}

const EARTH_RADIUS_M: f32 = 6356752.3;
const NEXRAD_RADAR_RANGE_M: f32 = 230000.0;

const RENDER_RATIO_TO_M: f32 = 0.00001; // every 1.0 in the render == 1.0/RENDER_RATIO_TO_M meters

const BELOW_THRESHOLD: f32 = 999.0;
const MOMENT_FOLDED: f32 = 998.0;

async fn execute(site: &str, date: &NaiveDate, time: &NaiveTime) -> Result<()> {
    let mut window = Window::new("NEXRAD Volumetric Renderer");

    let earth_scaled_radius = EARTH_RADIUS_M * RENDER_RATIO_TO_M;
    let mut earth = window.add_sphere(earth_scaled_radius);
    earth.set_local_translation(Translation3::new(0.0, -earth_scaled_radius, 0.0));
    earth.set_color(82.0 / 255.0, 143.0 / 255.0, 79.0 / 255.0);

    let nexrad_radar_diameter_scaled = NEXRAD_RADAR_RANGE_M * RENDER_RATIO_TO_M;
    let mut range_indicator = window.add_cylinder(nexrad_radar_diameter_scaled, 0.01);
    range_indicator.set_color(255.0 / 255.0, 140.0 / 255.0, 117.0 / 255.0);

    window.set_light(Light::Absolute(Point3::new(0.0, 100.0, 0.0)));

    let files = list_files(site, date).await?;
    if files.is_empty() {
        panic!("No files found for date/site");
    }

    let file = nearest_file(&files, time);
    println!("Nearest file: {}", file.identifier());

    let data = download_file(file).await?;
    println!("Downloaded file has {} bytes.", data.len());

    let decompressed_data = decompress_file(&data)?;
    println!("Decompressed file has {} bytes.", decompressed_data.len());

    let decoded = decode_file(&decompressed_data)?;
    println!(
        "Decoded file has {} elevation scans.",
        decoded.elevation_scans().len()
    );

    let mut elevation_scans: Vec<_> = decoded.elevation_scans().iter().collect();
    elevation_scans.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    let (_, radials) = elevation_scans.first().expect("has elevation scan");

    let radial = radials.iter().next().unwrap();
    let radial_reflectivity = radial.reflectivity_data().unwrap().data();

    let gate_interval_m = radial_reflectivity.data_moment_range_sample_interval() as f32;

    for radial in *radials {
        let mut azimuth_angle = radial.header().azm() - 90.0;
        if azimuth_angle < 0.0 {
            azimuth_angle = 360.0 + azimuth_angle;
        }

        let azimuth_spacing = radial.header().azm_res() as f32;

        let mut azimuth = azimuth_angle.floor();
        if (azimuth_angle + azimuth_spacing).floor() > azimuth {
            azimuth += azimuth_spacing;
        }

        let start_angle = azimuth * (PI / 180.0);

        let mut distance_m = gate_interval_m;

        let data_moment = radial.reflectivity_data().unwrap();

        let mut raw_gates: Vec<u16> =
            vec![0; data_moment.data().number_data_moment_gates() as usize];

        assert_eq!(data_moment.data().data_word_size(), 8);
        for (i, v) in data_moment.moment_data().iter().enumerate() {
            raw_gates[i] = *v as u16;
        }

        let mut scaled_gates: Vec<f32> = Vec::new();
        for raw_gate in raw_gates {
            if raw_gate == 0 {
                scaled_gates.push(BELOW_THRESHOLD);
            } else if raw_gate == 1 {
                scaled_gates.push(MOMENT_FOLDED);
            } else {
                let scale = data_moment.data().scale();
                let offset = data_moment.data().offset();

                let scaled_gate = if scale == 0.0 {
                    raw_gate as f32
                } else {
                    (raw_gate as f32 - offset) / scale
                };

                scaled_gates.push(scaled_gate);
            }
        }

        for (i, scaled_gate) in scaled_gates.iter().enumerate() {
            if i % 10 != 0 {
                continue;
            }

            let scaled_gate = *scaled_gate;

            if scaled_gate != BELOW_THRESHOLD {
                let angle_cos = start_angle.cos();
                let angle_sin = start_angle.sin();

                let position_x = (angle_cos * distance_m) * RENDER_RATIO_TO_M;
                let position_y = (angle_sin * distance_m) * RENDER_RATIO_TO_M;

                let mut pixel = window.add_cube(0.005, 0.002, 0.002);
                pixel.set_local_translation(Translation3::new(position_x, 0.2, position_y));

                let color = if scaled_gate < 5.0 || scaled_gate == BELOW_THRESHOLD {
                    (0, 0, 0)
                } else if scaled_gate >= 5.0 && scaled_gate < 10.0 {
                    (0x40, 0xe8, 0xe3)
                } else if scaled_gate >= 10.0 && scaled_gate < 15.0 {
                    (0x26, 0xa4, 0xfa)
                } else if scaled_gate >= 15.0 && scaled_gate < 20.0 {
                    (0x00, 0x30, 0xed)
                } else if scaled_gate >= 20.0 && scaled_gate < 25.0 {
                    (0x49, 0xfb, 0x3e)
                } else if scaled_gate >= 25.0 && scaled_gate < 30.0 {
                    (0x36, 0xc2, 0x2e)
                } else if scaled_gate >= 30.0 && scaled_gate < 35.0 {
                    (0x27, 0x8c, 0x1e)
                } else if scaled_gate >= 35.0 && scaled_gate < 40.0 {
                    (0xfe, 0xf5, 0x43)
                } else if scaled_gate >= 40.0 && scaled_gate < 45.0 {
                    (0xeb, 0xb4, 0x33)
                } else if scaled_gate >= 45.0 && scaled_gate < 50.0 {
                    (0xf6, 0x95, 0x2e)
                } else if scaled_gate >= 50.0 && scaled_gate < 55.0 {
                    (0xf8, 0x0a, 0x26)
                } else if scaled_gate >= 55.0 && scaled_gate < 60.0 {
                    (0xcb, 0x05, 0x16)
                } else if scaled_gate >= 60.0 && scaled_gate < 65.0 {
                    (0xa9, 0x08, 0x13)
                } else if scaled_gate >= 65.0 && scaled_gate < 70.0 {
                    (0xee, 0x34, 0xfa)
                } else {
                    (0xff, 0xff, 0xFF)
                };

                pixel.set_color(
                    color.0 as f32 / 255.0,
                    color.1 as f32 / 255.0,
                    color.2 as f32 / 255.0,
                );
            }

            distance_m += gate_interval_m;
            azimuth += azimuth_spacing;
        }
    }

    while window.render() {}

    Ok(())
}

fn nearest_file<'a>(files: &'a Vec<FileMetadata>, time: &NaiveTime) -> &'a FileMetadata {
    let mut nearest = files.first().unwrap();

    let get_file_time = |file: &FileMetadata| {
        let identifier_parts = file.identifier().split('_').collect::<Vec<&str>>();
        let identifier_time = identifier_parts[1];
        NaiveTime::parse_from_str(identifier_time, "%H%M%S").unwrap()
    };

    let mut nearest_diff = time.signed_duration_since(get_file_time(nearest));
    for file in files {
        let diff = time.signed_duration_since(get_file_time(file));
        if diff < nearest_diff {
            nearest = file;
            nearest_diff = diff;
        }
    }

    nearest
}
