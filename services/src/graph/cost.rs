use crate::osm::{Cycleway, Distance, Road, WayLabels};
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

/// Wire format for incoming cost model JSON — uses HashMaps keyed by enum name.
/// Converted to CostModel (array-backed) before use.
#[derive(Deserialize)]
struct CostModelInput {
    cycleway_coefficient: Cost,
    road_coefficient: Cost,
    salmon_coefficient: Cost,
    #[serde(default)]
    distance_coefficient: Cost,
    #[serde(default)]
    elevation_coefficient: Cost,
    cycleway_weights: HashMap<Cycleway, Cost>,
    road_weights: HashMap<Road, Cost>,
}

/// Cost model with array-backed weight lookups.
/// Cycleway and Road are #[repr(u8)] enums, so weights[variant as usize] is a direct
/// array index — no HashMap overhead in the hot path.
#[derive(Debug)]
pub struct CostModel {
    cycleway_coefficient: Cost,
    road_coefficient: Cost,
    salmon_coefficient: Cost,
    /// Base per-meter cost added to every segment, independent of road type.
    /// When non-zero, drives the algorithm toward shorter routes.
    distance_coefficient: Cost,
    /// Controls hill avoidance. 0.0 = ignore elevation, higher = avoid hills more.
    elevation_coefficient: Cost,
    /// Indexed by Cycleway discriminant (No=0, Shared=1, Lane=2, Track=3)
    cycleway_weights: [Cost; 4],
    /// Indexed by Road discriminant (Pedestrian=0, Bike=1, Local=2, Collector=3, Arterial=4)
    road_weights: [Cost; 5],
}

impl From<CostModelInput> for CostModel {
    fn from(input: CostModelInput) -> Self {
        let mut cycleway_weights = [0.0f32; 4];
        for (variant, weight) in &input.cycleway_weights {
            cycleway_weights[*variant as usize] = *weight;
        }

        let mut road_weights = [0.0f32; 5];
        for (variant, weight) in &input.road_weights {
            road_weights[*variant as usize] = *weight;
        }

        Self {
            cycleway_coefficient: input.cycleway_coefficient,
            road_coefficient: input.road_coefficient,
            salmon_coefficient: input.salmon_coefficient,
            distance_coefficient: input.distance_coefficient,
            elevation_coefficient: input.elevation_coefficient,
            cycleway_weights,
            road_weights,
        }
    }
}

impl<'de> Deserialize<'de> for CostModel {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(CostModelInput::deserialize(deserializer)?.into())
    }
}

impl Default for CostModel {
    fn default() -> Self {
        let mut cycleway_weights = [0.0f32; 4];
        cycleway_weights[Cycleway::No as usize] = 1.7;
        cycleway_weights[Cycleway::Shared as usize] = 1.5;
        cycleway_weights[Cycleway::Lane as usize] = 1.0;
        cycleway_weights[Cycleway::Track as usize] = 0.5;

        let mut road_weights = [0.0f32; 5];
        road_weights[Road::Pedestrian as usize] = 1.2;
        road_weights[Road::Bike as usize] = 0.5;
        road_weights[Road::Local as usize] = 1.2;
        road_weights[Road::Collector as usize] = 1.4;
        road_weights[Road::Arterial as usize] = 2.0;

        Self {
            cycleway_coefficient: 0.3,
            road_coefficient: 0.4,
            salmon_coefficient: 1.3,
            distance_coefficient: 0.0,
            elevation_coefficient: 0.0,
            cycleway_weights,
            road_weights,
        }
    }
}

impl CostModel {
    #[inline]
    pub fn calculate_cost(&self, way_labels: &WayLabels) -> Cost {
        let (cycleway, road, salmon) = way_labels;
        let cycleway_cost = self.cycleway_coefficient * self.cycleway_weights[*cycleway as usize];
        let road_cost = self.road_coefficient * self.road_weights[*road as usize];
        let salmon_cost = if *salmon {
            self.salmon_coefficient
        } else {
            1.0
        };
        (cycleway_cost + road_cost + self.distance_coefficient) * salmon_cost
    }

    /// Additive elevation cost for a segment.
    ///
    /// Starts from a baseline cost per meter, then:
    /// - Uphill: adds quadratic grade penalty on top of baseline
    /// - Downhill: subtracts a tapered benefit from baseline (never below 0)
    /// - Flat / unknown: pays just the baseline
    ///
    /// This offset approach means downhill segments are cheaper than flat but
    /// never free, and segments with no elevation data (bridges) aren't
    /// artificially preferred.
    #[inline]
    pub fn calculate_elevation_cost(
        &self,
        elevation_gain: i16,
        elevation_loss: i16,
        distance: Distance,
    ) -> Cost {
        if self.elevation_coefficient == 0.0 || distance <= 0 {
            return 0.0;
        }

        // Baseline: every segment pays a flat per-meter elevation-awareness cost.
        // This is the "neutral" cost that flat segments pay.
        let baseline = 0.1 * self.elevation_coefficient * distance as f32;

        // Sentinel: -1 means no elevation data (bridges, over water).
        // Penalize above baseline — these likely involve a climb we can't measure.
        if elevation_gain < 0 {
            return baseline + 0.1 * self.elevation_coefficient * distance as f32;
        }

        let mut cost = baseline;

        // Uphill penalty: quadratic in grade, added on top of baseline
        if elevation_gain > 0 {
            let grade = elevation_gain as f32 / distance as f32;
            cost += self.elevation_coefficient * grade * grade * distance as f32;
        }

        // Downhill benefit: tapered reduction from baseline
        // Gentle descents give most benefit; steep descents plateau
        // Clamped so cost never goes below 0
        if elevation_loss > 0 {
            let grade = elevation_loss as f32 / distance as f32;
            let taper = 1.0 - (-15.0 * grade).exp(); // saturates around grade ~0.15
            let benefit = baseline * taper; // at most removes the full baseline
            cost -= benefit;
        }

        cost.max(0.0)
    }
}
