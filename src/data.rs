use crate::result::Result;
use crate::state::ProcessingStatistics;
use crate::RENDER_RATIO_TO_M;
use chrono::{NaiveDate, NaiveTime};
use nexrad::decode::decode_file;
use nexrad::decompress::decompress_file;
use nexrad::download::{download_file, list_files};
use nexrad::file::FileMetadata;
use nexrad::model::DataFile;
use std::f32::consts::PI;
use std::time::Instant;
use three_d::Vector3;

pub const BELOW_THRESHOLD: f32 = 999.0;
pub const MOMENT_FOLDED: f32 = 998.0;

type RgbColor = (u8, u8, u8);

#[derive(Clone)]
pub struct ColoredPoint {
    pub pos: Vector3<f32>,
    pub strength: f32,
    pub raw: RgbColor,
    pub density: RgbColor,
    pub hybrid: RgbColor,
}

impl ColoredPoint {
    fn new(pos: Vector3<f32>, strength: f32) -> Self {
        Self {
            pos,
            strength,
            raw: (0, 0, 0),
            density: (0, 0, 0),
            hybrid: (0, 0, 0),
        }
    }
}

pub async fn get_data(
    site: &str,
    date: &NaiveDate,
    time: &NaiveTime,
    stats: &mut ProcessingStatistics,
) -> Result<DataFile> {
    let load_start = Instant::now();

    let files = list_files(site, date).await?;
    if files.is_empty() {
        panic!("No files found for date/site");
    }

    let file = nearest_file(&files, time);
    println!("Nearest file: {}", file.identifier());

    let data = if !std::path::Path::new(&file.identifier()).exists() {
        println!("Downloading file...");
        let downloaded_data = download_file(file).await?;

        println!("Writing file to disk...");
        std::fs::write(&file.identifier(), &downloaded_data)?;

        downloaded_data
    } else {
        println!("File already exists on disk, skipping download.");
        std::fs::read(&file.identifier())?
    };

    stats.load_ms = load_start.elapsed().as_millis();

    let decompress_start = Instant::now();
    let decompressed_data = decompress_file(&data)?;
    stats.decompress_ms = decompress_start.elapsed().as_millis();
    println!("Decompressed file has {} bytes.", decompressed_data.len());

    let decode_start = Instant::now();
    let decoded = decode_file(&decompressed_data)?;
    stats.decode_ms = decode_start.elapsed().as_millis();
    println!(
        "Decoded file has {} elevation scans.",
        decoded.elevation_scans().len()
    );

    Ok(decoded)
}

pub fn nearest_file<'a>(files: &'a Vec<FileMetadata>, time: &NaiveTime) -> &'a FileMetadata {
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

pub fn get_points(data: &DataFile, threshold: f32) -> Vec<ColoredPoint> {
    let mut points: Vec<ColoredPoint> = Vec::new();

    for (elevation, radials) in data.elevation_scans() {
        for radial in radials {
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

            let data_moment = radial.reflectivity_data().unwrap();
            let mut distance_m = data_moment.data().data_moment_range_sample_interval() as f32;

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

            for scaled_gate in scaled_gates {
                if scaled_gate != BELOW_THRESHOLD && scaled_gate > threshold {
                    let scaled_distance = distance_m * RENDER_RATIO_TO_M;
                    let position_x = start_angle.cos() * scaled_distance;
                    let position_y = start_angle.sin() * scaled_distance;
                    let position_z = (*elevation as f32 * (PI / 180.0)).sin() * scaled_distance;

                    points.push(ColoredPoint::new(
                        Vector3::new(position_x, position_z, position_y),
                        scaled_gate,
                    ));
                }

                distance_m += data_moment.data().data_moment_range_sample_interval() as f32;
                azimuth += azimuth_spacing;
            }
        }
    }

    points
}
