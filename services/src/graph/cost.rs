use crate::osm::{Cycleway, Distance, Road, WayLabels};
use serde::{Deserialize, Serializer};
use std::collections::HashMap;

/// Lerp helper: blend between two values by t ∈ [0, 1].
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Lerp two fixed-size arrays element-wise.
fn lerp_arr<const N: usize>(a: &[f32; N], b: &[f32; N], t: f32) -> [f32; N] {
    let mut out = [0.0f32; N];
    for i in 0..N {
        out[i] = lerp(a[i], b[i], t);
    }
    out
}

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

    /// Dimensionless elevation multiplier for a segment.
    ///
    /// Returns a value ≥ 0 that is used multiplicatively:
    ///   segment_cost = cost_factor * length * (1.0 + elevation_multiplier)
    ///
    /// - Flat / elevation disabled: 0.0 (no change to base cost)
    /// - Uphill: positive, quadratic in grade
    /// - Downhill: negative (tapered), reducing base cost but never below 0
    /// - Unknown elevation (bridges): small positive penalty
    #[inline]
    pub fn calculate_elevation_multiplier(
        &self,
        elevation_gain: i16,
        elevation_loss: i16,
        distance: Distance,
    ) -> Cost {
        if self.elevation_coefficient == 0.0 || distance <= 0 {
            return 0.0;
        }

        // Baseline: flat segments pay a small awareness cost
        let baseline = 0.1 * self.elevation_coefficient;

        // Sentinel: -1 means no elevation data (bridges, over water).
        // Penalize above baseline — these likely involve a climb we can't measure.
        if elevation_gain < 0 {
            return baseline + 0.1 * self.elevation_coefficient;
        }

        let mut multiplier = baseline;

        // Uphill penalty: quadratic in grade
        if elevation_gain > 0 {
            let grade = elevation_gain as f32 / distance as f32;
            multiplier += self.elevation_coefficient * grade * grade;
        }

        // Downhill benefit: tapered reduction from baseline
        // Gentle descents give most benefit; steep descents plateau
        // Clamped so multiplier never goes below 0
        if elevation_loss > 0 {
            let grade = elevation_loss as f32 / distance as f32;
            let taper = 1.0 - (-15.0 * grade).exp(); // saturates around grade ~0.15
            let benefit = baseline * taper;
            multiplier -= benefit;
        }

        multiplier.max(0.0)
    }
}

// ---------------------------------------------------------------------------
// Mobile cost model — high-level parameterization that resolves to CostModel
// ---------------------------------------------------------------------------

/// Pre-built speed profile: minimize distance, road/cycleway type matters little.
const SPEED_PROFILE: CostModelProfile = CostModelProfile {
    cycleway_coefficient: 0.1,
    road_coefficient: 0.1,
    distance_coefficient: 0.5,
    cycleway_weights: [1.2, 1.1, 1.0, 0.9], // No, Shared, Lane, Track — nearly flat
    road_weights: [1.1, 0.9, 1.0, 1.1, 1.3], // Ped, Bike, Local, Collector, Arterial — mild spread
};

/// Pre-built comfort profile: strongly prefer protected infrastructure.
const COMFORT_PROFILE: CostModelProfile = CostModelProfile {
    cycleway_coefficient: 0.7,
    road_coefficient: 0.5,
    distance_coefficient: 0.0,
    cycleway_weights: [1.7, 1.5, 1.0, 0.5], // No, Shared, Lane, Track — strong spread
    road_weights: [0.9, 0.5, 1.2, 1.4, 2.0], // Ped, Bike, Local, Collector, Arterial
};

struct CostModelProfile {
    cycleway_coefficient: Cost,
    road_coefficient: Cost,
    distance_coefficient: Cost,
    cycleway_weights: [Cost; 4],
    road_weights: [Cost; 5],
}

/// Mobile-optimized cost model: a few intuitive controls that resolve
/// to the full CostModel used by the traversal engine.
#[derive(Debug, Deserialize)]
pub struct MobileCostModel {
    /// 0.0 = pure speed, 1.0 = pure comfort
    priority: f32,
    /// 0 = ignore hills, 1 = avoid, 2 = strongly avoid
    hill_penalty: u8,
    /// 0 = ignore salmon, 1 = avoid, 2 = strongly avoid
    salmon_penalty: u8,
    /// When true, further penalizes arterials and collectors
    #[serde(default)]
    avoid_major_roads: bool,
}

impl MobileCostModel {
    /// Resolve this high-level model into the concrete CostModel used by A*.
    pub fn resolve(self) -> CostModel {
        let t = self.priority.clamp(0.0, 1.0);

        let cycleway_coefficient = lerp(
            SPEED_PROFILE.cycleway_coefficient,
            COMFORT_PROFILE.cycleway_coefficient,
            t,
        );
        let road_coefficient = lerp(
            SPEED_PROFILE.road_coefficient,
            COMFORT_PROFILE.road_coefficient,
            t,
        );
        let distance_coefficient = lerp(
            SPEED_PROFILE.distance_coefficient,
            COMFORT_PROFILE.distance_coefficient,
            t,
        );

        let cycleway_weights = lerp_arr(
            &SPEED_PROFILE.cycleway_weights,
            &COMFORT_PROFILE.cycleway_weights,
            t,
        );
        let mut road_weights = lerp_arr(
            &SPEED_PROFILE.road_weights,
            &COMFORT_PROFILE.road_weights,
            t,
        );

        // Avoid major roads: bump arterial and collector weights
        if self.avoid_major_roads {
            road_weights[Road::Arterial as usize] += 1.0;
            road_weights[Road::Collector as usize] += 0.3;
        }

        let salmon_coefficient = match self.salmon_penalty {
            0 => 1.1,
            1 => 1.35,
            _ => 2.5,
        };

        let elevation_coefficient = match self.hill_penalty {
            0 => 0.0,
            1 => 1.0,
            _ => 2.5,
        };

        CostModel {
            cycleway_coefficient,
            road_coefficient,
            salmon_coefficient,
            distance_coefficient,
            elevation_coefficient,
            cycleway_weights,
            road_weights,
        }
    }
}
