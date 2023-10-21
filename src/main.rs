use chrono::{NaiveDate, NaiveTime};
use nexrad::decode::decode_file;
use nexrad::decompress::decompress_file;
use nexrad::download::{download_file, list_files};
use nexrad::file::FileMetadata;
use nexrad::model::DataFile;
use std::f32::consts::PI;
use three_d::{
    degrees, vec3, Camera, ClearState, ColorMaterial, Context, CpuMaterial, CpuMesh,
    DirectionalLight, FrameOutput, Gm, InstancedMesh, Mat4, Mesh, OrbitControl, PhysicalMaterial,
    PointCloud, Positions, Srgba, Vec3, Vector3, Window, WindowSettings,
};

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
    let sampled_points = points.iter().step_by(10).collect::<Vec<_>>();
    println!("Scan contains {} points.", sampled_points.len());

    let point_cloud_gm = get_point_cloud_object(&context, sampled_points);

    let (mut camera, mut control) = get_camera_and_control(&window);

    let mut angle_deg = 0.0;
    window.render_loop(move |mut frame_input| {
        camera.set_viewport(frame_input.viewport);
        control.handle_events(&mut camera, &mut frame_input.events);
        update_camera(&mut angle_deg, &mut camera);

        let objects = point_cloud_gm
            .into_iter()
            .chain(&earth)
            .chain(&radar_indicator);

        frame_input
            .screen()
            .clear(ClearState::color_and_depth(0.0, 0.0, 0.0, 1.0, 1.0))
            .render(&camera, objects, &[&sun]);

        FrameOutput::default()
    });

    Ok(())
}

async fn get_data(site: &str, date: &NaiveDate, time: &NaiveTime) -> Result<DataFile> {
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

    let decompressed_data = decompress_file(&data)?;
    println!("Decompressed file has {} bytes.", decompressed_data.len());

    let decoded = decode_file(&decompressed_data)?;
    println!(
        "Decoded file has {} elevation scans.",
        decoded.elevation_scans().len()
    );

    Ok(decoded)
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

fn get_points(data: &DataFile, threshold: f32) -> Vec<(Vector3<f32>, (u8, u8, u8))> {
    let mut points = Vec::new();

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

                    points.push((Vector3::new(position_x, position_z, position_y), color));
                }

                distance_m += data_moment.data().data_moment_range_sample_interval() as f32;
                azimuth += azimuth_spacing;
            }
        }
    }

    points
}

fn get_camera_and_control(window: &Window) -> (Camera, OrbitControl) {
    let camera = Camera::new_perspective(
        window.viewport(),
        vec3(0.0, 2.0, 5.0),
        vec3(0.0, 0.0, -5.0),
        vec3(0.0, 1.0, 0.0),
        degrees(45.0),
        0.1,
        1000.0,
    );

    let control = OrbitControl::new(Vec3::new(0.0, 0.0, 0.0), 0.01, 5.0);

    (camera, control)
}

fn get_sun_light(context: &Context) -> DirectionalLight {
    DirectionalLight::new(context, 1.0, Srgba::WHITE, &vec3(0.0, -1.0, -1.0))
}

fn get_earth_object(context: &Context) -> Gm<Mesh, PhysicalMaterial> {
    let earth_scaled_radius = EARTH_RADIUS_M * RENDER_RATIO_TO_M;

    let mut earth = Gm::new(
        Mesh::new(context, &CpuMesh::sphere(100)),
        PhysicalMaterial::new_opaque(
            context,
            &CpuMaterial {
                albedo: Srgba {
                    r: 40,
                    g: 100,
                    b: 40,
                    a: 255,
                },
                ..Default::default()
            },
        ),
    );

    earth.set_transformation(
        Mat4::from_translation(vec3(0.0, -earth_scaled_radius, 0.0))
            * Mat4::from_scale(earth_scaled_radius),
    );

    earth
}

fn get_radar_indicator_object(context: &Context) -> Gm<Mesh, PhysicalMaterial> {
    let nexrad_radar_diameter_scaled = NEXRAD_RADAR_RANGE_M * RENDER_RATIO_TO_M;

    let mut radar_indicator = Gm::new(
        Mesh::new(context, &CpuMesh::cylinder(100)),
        PhysicalMaterial::new_opaque(
            context,
            &CpuMaterial {
                albedo: Srgba {
                    r: 255,
                    g: 0,
                    b: 0,
                    a: 255,
                },
                ..Default::default()
            },
        ),
    );

    radar_indicator.set_transformation(
        Mat4::from_translation(vec3(0.0, 0.0, 0.0))
            * Mat4::from_angle_z(degrees(90.0))
            * Mat4::from_nonuniform_scale(
                0.01,
                nexrad_radar_diameter_scaled,
                nexrad_radar_diameter_scaled,
            ),
    );

    radar_indicator
}

fn get_point_cloud_object(
    context: &Context,
    points: Vec<&(Vector3<f32>, (u8, u8, u8))>,
) -> Gm<InstancedMesh, ColorMaterial> {
    let mut point_cloud = PointCloud::default();
    point_cloud.positions = Positions::F32(
        points
            .iter()
            .map(|(p, _)| vec3(p.x, p.y, p.z))
            .collect::<Vec<_>>(),
    );
    point_cloud.colors = Some(
        points
            .iter()
            .map(|(_, c)| Srgba::new(c.0, c.1, c.2, 255))
            .collect::<Vec<_>>(),
    );

    let mut point_mesh = CpuMesh::sphere(4);
    point_mesh.transform(&Mat4::from_scale(0.002)).unwrap();

    let point_cloud_gm = Gm {
        geometry: InstancedMesh::new(&context, &point_cloud.into(), &point_mesh),
        material: ColorMaterial::default(),
    };

    point_cloud_gm
}

fn update_camera(angle_deg: &mut f64, camera: &mut Camera) {
    *angle_deg += 0.2;
    if *angle_deg > 360.0 {
        *angle_deg = 0.0;
    }

    let angle = *angle_deg as f32 * (PI / 180.0);
    let position_x = angle.cos() * NEXRAD_RADAR_RANGE_M * RENDER_RATIO_TO_M * 1.5;
    let position_y = angle.sin() * NEXRAD_RADAR_RANGE_M * RENDER_RATIO_TO_M * 1.5;
    camera.set_view(
        Vec3::new(position_x, 2.0, position_y),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
    );
}
