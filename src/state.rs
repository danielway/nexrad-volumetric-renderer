use crate::ColoredPoint;

pub struct State {
    pub processing: bool,
    pub points: Option<Vec<ColoredPoint>>,
    pub statistics: Option<ProcessingStatistics>,
}

#[derive(Default)]
pub struct ProcessingStatistics {
    pub load_ms: u128,
    pub decompress_ms: u128,
    pub decode_ms: u128,
    pub pointing_ms: u128,
    pub sampling_ms: u128,
    pub coloring_ms: u128,
}
