use std::collections::HashMap;

use crate::osm::{Cycleway, Distance, Road, WayId, WayLabels};

use super::Graph;

pub type Cost = f32;
pub type Weight = f32;

pub struct CostModelFactors {
    pub cycleway: Cycleway,
}

pub struct CostModel {
    cycleway_coefficient: Cost,
    road_coefficient: Cost,
    salmon_coefficient: Cost,

    cycleway_weights: HashMap<Cycleway, Weight>,
    road_weights: HashMap<Road, Weight>,
}

// TODO: expose this configuration to FE
impl Default for CostModel {
    fn default() -> Self {
        Self {
            cycleway_coefficient: 0.3,
            road_coefficient: 0.4,
            salmon_coefficient: 0.3,

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
    #[inline]
    pub fn calculate_cost(&self, graph: &Graph, way: WayId, length: Distance) -> Result<(Cost, WayLabels), anyhow::Error> {
        let labels = graph.get_way_labels(way)?;
        let (ref cycleway, ref road, salmon) = labels;

        let cycleway_cost = self.cycleway_coefficient * self.cycleway_weights.get(cycleway).unwrap();
        let road_cost = self.road_coefficient * self.road_weights.get(road).unwrap();
        let salmon_cost = if salmon { self.salmon_coefficient } else { 0.0 };

        let cost = (cycleway_cost + road_cost + salmon_cost) * length;

        Ok((cost, labels))
    }
}
