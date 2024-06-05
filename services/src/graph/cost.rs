use std::collections::HashMap;

use crate::osm::WayId;

use super::Graph;

pub type Cost = f32;
pub type Weight = f32;

#[derive(Hash, Eq, PartialEq)]
pub enum BikeLane {
    Protected,
    // Buffered,
    Designated,
    Shared,
}

pub struct CostModelFactors {
    pub bike_lane: BikeLane,
}

pub struct CostModel {
    // Weights
    bike_lane: HashMap<BikeLane, Weight>,
}

impl Default for CostModel {
    fn default() -> Self {
        Self {
            bike_lane: HashMap::from([
                (BikeLane::Protected, 0.5),
                (BikeLane::Designated, 1.0),
                (BikeLane::Shared, 2.0),
            ]),
        }
    }
}

impl CostModel {
    // TODO: should have its own reference to the DB Connection,
    // not leverage Graph helper functions
    // TODO: should be 3 phases / functions:
    //       1. collate CostModelFactors struct from DB info (or use memo)
    //       2. apply factors against this CostModel's weights
    //       3. sum and return cost
    pub fn calculate_cost(&self, graph: &Graph, way: WayId) -> Result<Cost, anyhow::Error> {
        let way_tags = graph.get_way_tags(way)?;

        // see https://wiki.openstreetmap.org/wiki/Bicycle
        // for now, checking both sides for tracks / lanes
        let bike_lane = match way_tags.get("cycleway:left").map_or("none", |v| v.as_str()) {
            "track" => BikeLane::Protected,
            "lane" => BikeLane::Designated,
            _ => match way_tags
                .get("cycleway:right")
                .map_or("none", |v| v.as_str())
            {
                "track" => BikeLane::Protected,
                "lane" => BikeLane::Designated,
                _ => BikeLane::Shared,
            },
        };

        // TODO: starting bike lane factor?
        // right now just a single cost...but different factors should have different starting
        // costs, and different values for each factor will have different weights
        let mut cost = 50.0;

        if let Some(weight) = self.bike_lane.get(&bike_lane) {
            cost *= weight;
        }

        Ok(cost)
    }
}
