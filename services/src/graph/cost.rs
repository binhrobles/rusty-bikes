use super::Graph;
use crate::osm::{Cycleway, Road, WayId, WayLabels};
use serde::{Deserialize, Serializer};
use std::collections::HashMap;
use tracing::error;

pub type Cost = f32;
pub type Weight = f32;

/// simple serialization of a Node to just its ID
pub fn serialize_cost_simple<S>(cost: &Cost, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_i64(cost.ceil() as i64)
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
    pub fn calculate_cost(&self, graph: &Graph, way: WayId) -> (Cost, WayLabels) {
        match graph.get_way_labels(way) {
            Err(e) => {
                // shouldn't error...but if we do just return an absurd cost
                error!("cost function: could not fetch way #{way}");
                error!("{e}");
                (1000.0, (Cycleway::Shared, Road::Arterial, true))
            }
            Ok(labels) => {
                let (ref cycleway, ref road, salmon) = labels;

                let cycleway_cost =
                    self.cycleway_coefficient * self.cycleway_weights.get(cycleway).unwrap();
                let road_cost = self.road_coefficient * self.road_weights.get(road).unwrap();
                let salmon_cost = if salmon { self.salmon_coefficient } else { 1.0 };

                let cost = (cycleway_cost + road_cost) * salmon_cost;

                (cost, labels)
            }
        }
    }
}
