use chrono::{NaiveDate, NaiveTime};

#[derive(Eq, PartialEq)]
pub enum InteractionMode {
    Orbit,
    ManualOrbit,
}

#[derive(Eq, PartialEq)]
pub enum PointWeightMode {
    ReturnStrength,
    Density,
    Hybrid,
}

#[derive(Eq, PartialEq)]
pub enum ClusteringMode {
    KNN,
    DBSCAN,
}

pub struct Parameters {
    pub site: String,
    pub date: NaiveDate,
    pub time: NaiveTime,
    pub interaction_mode: InteractionMode,
    pub data_sampling: u16,
    pub point_weight_mode: PointWeightMode,
    pub clustering_mode: ClusteringMode,
    pub clustering_threshold: f32,
}
