use crate::osm::{Cycleway, Road, WayLabels};
use serde::{Deserialize, Serializer};
use std::collections::HashMap;

pub type Cost = f32;
pub type Weight = f32;

/// simple serialization of an f32 to just an int
pub fn serialize_as_int<S>(float: &f32, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_i64(float.ceil() as i64)
}

/// truncates a float to 2 decimal places when serializing
pub fn serialize_float_rounded<S>(float: &f32, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_f64((*float as f64 * 100.0).trunc() / 100.0)
}

#[derive(Debug, Deserialize)]
pub struct CostModel {
    cycleway_coefficient: Cost,
    road_coefficient: Cost,
    salmon_coefficient: Cost,

    cycleway_weights: HashMap<Cycleway, Cost>,
    road_weights: HashMap<Road, Cost>,
}

impl Default for CostModel {
    fn default() -> Self {
        Self {
            cycleway_coefficient: 0.3,
            road_coefficient: 0.4,
            salmon_coefficient: 1.3,

            cycleway_weights: HashMap::from([
                (Cycleway::Track, 0.5),
                (Cycleway::Lane, 1.0),
                (Cycleway::Shared, 1.5),
                (Cycleway::No, 1.7),
            ]),
            road_weights: HashMap::from([
                (Road::Bike, 0.5),
                (Road::Pedestrian, 1.2),
                (Road::Local, 1.2),
                (Road::Collector, 1.4),
                (Road::Arterial, 2.0),
            ]),
        }
    }
}

impl CostModel {
    pub fn calculate_cost(&self, way_labels: &WayLabels) -> Cost {
        let (cycleway, road, salmon) = way_labels;

        let cycleway_cost =
            self.cycleway_coefficient * self.cycleway_weights.get(cycleway).unwrap();
        let road_cost = self.road_coefficient * self.road_weights.get(road).unwrap();
        let salmon_cost = if *salmon {
            self.salmon_coefficient
        } else {
            1.0
        };

        (cycleway_cost + road_cost) * salmon_cost
    }
}
