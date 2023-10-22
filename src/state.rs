use crate::ColoredPoint;

pub struct State {
    pub processing: bool,
    pub points: Option<Vec<ColoredPoint>>,
}
