use crate::data::{get_data, get_points, ColoredPoint, BELOW_THRESHOLD};
use crate::param::DataParams;
use crate::result::Result;
use crate::state::{ProcessingStatistics, State};
use dbscan::Classification;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

pub fn do_fetch_and_process(data_params: DataParams, state: Arc<Mutex<State>>) {
    tokio::spawn(async move {
        fetch_and_process(data_params, state)
            .await
            .expect("fetch and processes successfully");
    });
}

pub async fn fetch_and_process(data_params: DataParams, state: Arc<Mutex<State>>) -> Result<()> {
    {
        let mut state = state.lock().unwrap();

        if state.processing {
            panic!("cannot process concurrently")
        }

        state.processing = true;
    }

    let mut stats = ProcessingStatistics::default();

    let decoded = get_data(
        &data_params.site,
        &data_params.date,
        &data_params.time,
        &mut stats,
    )
    .await?;

    let pointing_start = Instant::now();
    let points = get_points(&decoded, 0.5);
    stats.pointing_ms = pointing_start.elapsed().as_millis();

    // Sample dataset to speed processing
    let mut sampled_points = points
        .into_iter()
        .step_by(data_params.sampling as usize)
        .collect::<Vec<_>>();
    println!("Scan contains {} points.", sampled_points.len());

    // todo: we need to weight and rescale geometrically before clustering
    // todo: in addition to result/density, weight should consider gate distance

    // println!("Clustering points...");
    // let (clusters, unclustered) = do_dbscan_clustering(sampled_points.clone());
    // println!(
    //     "Found {} clusters with {} remaining unclustered points.",
    //     clusters.len(),
    //     unclustered.len()
    // );

    // todo: coloring clusters to debug

    // let mut index = 0.0;
    // let mut get_color = || {
    //     let phi = (1.0 + 5.0_f64.sqrt()) / 2.0;
    //     let n = index * phi - (index * phi).floor();
    //     let color = HSL {
    //         h: n * 256.0,
    //         s: 1.0,
    //         l: 0.5,
    //     };
    //     index += 1.0;
    //     return color.to_rgb();
    // };
    //
    // println!("Recoloring clusters for debugging.");
    // let mut recolored_points = Vec::new();
    // let unclustered_color = (255, 255, 255);
    // for mut point in unclustered {
    //     point.1 = unclustered_color;
    //     recolored_points.push(point);
    // }
    // for cluster in clusters {
    //     let cluster_color = get_color();
    //     for mut point in cluster {
    //         point.1 = cluster_color;
    //         recolored_points.push(point);
    //     }
    // }

    color_points(&mut sampled_points);

    let mut state = state.lock().unwrap();
    state.points = Some(sampled_points);
    state.processing = false;
    state.statistics = Some(stats);

    println!("Done fetch/processing!");

    Ok(())
}

// Returns: (clustered points, unclustered points)
fn do_dbscan_clustering(points: Vec<ColoredPoint>) -> (Vec<Vec<ColoredPoint>>, Vec<ColoredPoint>) {
    let mut clusters = HashMap::new();
    let mut unclustered_points = Vec::new();

    let vectorized_points: Vec<Vec<f32>> = points
        .iter()
        .map(|p| vec![p.pos.x, p.pos.y, p.pos.z])
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

fn color_points(points: &mut Vec<ColoredPoint>) {
    for point in points {
        point.raw = if point.strength < 5.0 || point.strength == BELOW_THRESHOLD {
            (0, 0, 0)
        } else if point.strength >= 5.0 && point.strength < 10.0 {
            (0x40, 0xe8, 0xe3)
        } else if point.strength >= 10.0 && point.strength < 15.0 {
            (0x26, 0xa4, 0xfa)
        } else if point.strength >= 15.0 && point.strength < 20.0 {
            (0x00, 0x30, 0xed)
        } else if point.strength >= 20.0 && point.strength < 25.0 {
            (0x49, 0xfb, 0x3e)
        } else if point.strength >= 25.0 && point.strength < 30.0 {
            (0x36, 0xc2, 0x2e)
        } else if point.strength >= 30.0 && point.strength < 35.0 {
            (0x27, 0x8c, 0x1e)
        } else if point.strength >= 35.0 && point.strength < 40.0 {
            (0xfe, 0xf5, 0x43)
        } else if point.strength >= 40.0 && point.strength < 45.0 {
            (0xeb, 0xb4, 0x33)
        } else if point.strength >= 45.0 && point.strength < 50.0 {
            (0xf6, 0x95, 0x2e)
        } else if point.strength >= 50.0 && point.strength < 55.0 {
            (0xf8, 0x0a, 0x26)
        } else if point.strength >= 55.0 && point.strength < 60.0 {
            (0xcb, 0x05, 0x16)
        } else if point.strength >= 60.0 && point.strength < 65.0 {
            (0xa9, 0x08, 0x13)
        } else if point.strength >= 65.0 && point.strength < 70.0 {
            (0xee, 0x34, 0xfa)
        } else {
            (0xff, 0xff, 0xFF)
        };
    }
}
