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

#[derive(PartialEq, Clone)]
pub struct RenderParameters {
    pub interaction_mode: InteractionMode,
    pub point_color_mode: PointColorMode,
}

#[derive(Eq, PartialEq, Copy, Clone)]
pub enum ClusteringMode {
    KNN,
    DBSCAN,
}

#[derive(PartialEq, Clone)]
pub struct DataParameters {
    pub site: String,
    pub date: NaiveDate,
    pub time: NaiveTime,
    pub data_sampling: u16,
    pub clustering_mode: ClusteringMode,
    pub clustering_threshold: f32,
}
