use chrono::{NaiveDate, NaiveTime};
use kiss3d::light::Light;
use kiss3d::nalgebra::{Point3, Translation3, UnitQuaternion, Vector3};
use kiss3d::window::Window;
use nexrad::decode::decode_file;
use nexrad::decompress::decompress_file;
use nexrad::download::{download_file, list_files};
use nexrad::file::FileMetadata;

use crate::result::Result;

mod result;

const TARGET_SITE: &str = "KDMX";

#[tokio::main]
async fn main() {
    let target_date = NaiveDate::from_ymd_opt(2022, 3, 5).unwrap();
    let target_time = NaiveTime::from_hms_opt(23, 30, 0).unwrap();

    execute(TARGET_SITE, &target_date, &target_time).await.unwrap();
}

const EARTH_RADIUS_M: f64 = 6356752.3;
const RENDER_RATIO_TO_M: f64 = 0.001; // every 1.0 in the render == 1.0/RENDER_RATIO_TO_M meters

async fn execute(site: &str, date: &NaiveDate, time: &NaiveTime) -> Result<()> {
    let mut window = Window::new("NEXRAD Volumetric Renderer");

    let earth_scaled_radius = (EARTH_RADIUS_M * RENDER_RATIO_TO_M) as f32;
    let mut earth = window.add_sphere(earth_scaled_radius);
    earth.set_local_translation(Translation3::new(0.0, -earth_scaled_radius, 0.0));
    earth.set_color(82.0/255.0, 143.0/255.0, 79.0/255.0);

    let mut cube = window.add_cube(1.0, 1.0, 1.0);
    cube.set_color(1.0, 0.0, 0.0);
    cube.set_local_translation(Translation3::new(0.0, 1.0, 0.0));

    window.set_light(Light::Absolute(Point3::new(0.0, 100.0, 0.0)));

    let rotation = UnitQuaternion::from_axis_angle(&Vector3::y_axis(), 0.014);

    while window.render() {
        cube.prepend_to_local_rotation(&rotation);
    }

    return Ok(());

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
    println!("Decoded file has {} elevation scans.", decoded.elevation_scans().len());

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
