use crate::data::{get_data, get_points};
use crate::do_dbscan_clustering;
use crate::result::Result;
use crate::state::State;
use chrono::{NaiveDate, NaiveTime};
use hsl::HSL;
use std::sync::{Arc, Mutex};

pub fn do_fetch_and_process(
    site: String,
    date: NaiveDate,
    time: NaiveTime,
    state: Arc<Mutex<State>>,
) {
    tokio::spawn(async move {
        fetch_and_process(site, date, time, state)
            .await
            .expect("fetch and processes successfully");
    });
}

pub async fn fetch_and_process(
    site: String,
    date: NaiveDate,
    time: NaiveTime,
    state: Arc<Mutex<State>>,
) -> Result<()> {
    {
        let mut state = state.lock().unwrap();

        if state.processing {
            panic!("cannot process concurrently")
        }

        state.processing = true;
    }

    let decoded = get_data(&site, &date, &time).await?;
    let points = get_points(&decoded, 0.5);

    // Sample dataset to speed processing
    let sampled_points = points.into_iter().step_by(1000).collect::<Vec<_>>();
    println!("Scan contains {} points.", sampled_points.len());

    // todo: we need to weight and rescale geometrically before clustering
    // todo: in addition to result/density, weight should consider gate distance

    println!("Clustering points...");
    let (clusters, unclustered) = do_dbscan_clustering(sampled_points.clone());
    println!(
        "Found {} clusters with {} remaining unclustered points.",
        clusters.len(),
        unclustered.len()
    );

    // todo: coloring clusters to debug

    let mut index = 0.0;
    let mut get_color = || {
        let phi = (1.0 + 5.0_f64.sqrt()) / 2.0;
        let n = index * phi - (index * phi).floor();
        let color = HSL {
            h: n * 256.0,
            s: 1.0,
            l: 0.5,
        };
        index += 1.0;
        return color.to_rgb();
    };

    println!("Recoloring clusters for debugging.");
    let mut recolored_points = Vec::new();
    let unclustered_color = (255, 255, 255);
    for mut point in unclustered {
        point.1 = unclustered_color;
        recolored_points.push(point);
    }
    for cluster in clusters {
        let cluster_color = get_color();
        for mut point in cluster {
            point.1 = cluster_color;
            recolored_points.push(point);
        }
    }

    let mut state = state.lock().unwrap();
    state.points = Some(recolored_points);
    state.processing = false;

    println!("Done fetch/processing!");

    Ok(())
}
