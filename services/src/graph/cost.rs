use std::collections::HashMap;

use crate::osm::{Cycleway, Distance, Road, WayId, WayLabels};

use super::Graph;

// TODO: int?
pub type Cost = f32;
pub type Weight = f32;

pub struct CostModelFactors {
    pub cycleway: Cycleway,
}

pub struct CostModel {
    // Weights
    cycleway: HashMap<Cycleway, Weight>,
    road: HashMap<Road, Weight>,
}

impl Default for CostModel {
    fn default() -> Self {
        Self {
            cycleway: HashMap::from([
                (Cycleway::Track, 0.5),
                (Cycleway::Lane, 1.0),
                (Cycleway::Shared, 2.0),
            ]),
            road: HashMap::from([
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
    // TODO: should have its own reference to the DB Connection,
    // not leverage Graph helper functions
    #[inline]
    pub fn calculate_cost(&self, graph: &Graph, way: WayId, length: Distance) -> Result<(Cost, WayLabels), anyhow::Error> {
        let labels = graph.get_way_labels(way)?;
        let (ref _cycleway, ref road, ref _salmon) = labels;

        // let mut cycleway_cost = 25.0;
        // if let Some(weight) = self.cycleway.get(&cycleway) {
        //     cycleway_cost *= weight;
        // }

        let mut road_cost = 0.5;
        if let Some(weight) = self.road.get(road) {
            road_cost *= weight;
        }

        let cost = road_cost * length;

        Ok((cost, labels))
    }
}
