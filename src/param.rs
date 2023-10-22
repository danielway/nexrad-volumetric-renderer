use chrono::{NaiveDate, NaiveTime};

#[derive(Eq, PartialEq, Copy, Clone)]
pub enum InteractionMode {
    Orbit,
    ManualOrbit,
}

#[derive(Eq, PartialEq, Copy, Clone)]
pub enum PointColorMode {
    Raw,
    Density,
    Hybrid,
}

#[derive(Eq, PartialEq, Copy, Clone)]
pub enum ClusteringMode {
    KNN,
    DBSCAN,
}

#[derive(Clone)]
pub struct Parameters {
    pub site: String,
    pub date: NaiveDate,
    pub time: NaiveTime,
    pub interaction_mode: InteractionMode,
    pub data_sampling: u16,
    pub point_color_mode: PointColorMode,
    pub clustering_mode: ClusteringMode,
    pub clustering_threshold: f32,
}
