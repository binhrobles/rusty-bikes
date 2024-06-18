use super::Element;
use crate::osm::{Cycleway, Road, Salmoning, WayId};

/// Provides helper methods for interpreting and labeling the relevant OSM tags for bike routing
#[derive(Debug)]
pub struct OSMMapper {
    pub way: WayId,
    pub highway: String,
    pub bicycle: String,
    pub oneway: String,
    pub cycleway_right: String,
    pub cycleway_left: String,
    pub cycleway_both: String,
    pub cycleway_right_oneway: String,
    pub cycleway_left_oneway: String,
    pub oneway_bicycle: String,
}

/// Parses out the tags we want from this JSON map
impl From<&Element> for OSMMapper {
    fn from(element: &Element) -> Self {
        let tags = &element.tags;
        let highway = tags.get("highway").cloned().unwrap_or("none".to_owned());
        let bicycle = tags.get("bicycle").cloned().unwrap_or("none".to_owned());
        let oneway = tags.get("oneway").cloned().unwrap_or("none".to_owned());
        let cycleway_right = tags
            .get("cycleway:right")
            .cloned()
            .unwrap_or("none".to_owned());
        let cycleway_left = tags
            .get("cycleway:left")
            .cloned()
            .unwrap_or("none".to_owned());
        let cycleway_both = tags
            .get("cycleway:both")
            .cloned()
            .unwrap_or("none".to_owned());
        let cycleway_right_oneway = tags
            .get("cycleway:right:oneway")
            .cloned()
            .unwrap_or("none".to_owned());
        let cycleway_left_oneway = tags
            .get("cycleway:left:oneway")
            .cloned()
            .unwrap_or("none".to_owned());
        let oneway_bicycle = tags
            .get("oneway:bicycle")
            .cloned()
            .unwrap_or("none".to_owned());

        OSMMapper {
            way: element.id,
            highway,
            bicycle,
            oneway,
            cycleway_right,
            cycleway_left,
            cycleway_both,
            cycleway_right_oneway,
            cycleway_left_oneway,
            oneway_bicycle,
        }
    }
}

impl OSMMapper {
    /// Given these OSM tags, calculate road label
    pub fn get_road_label(&self) -> Road {
        match self.highway.as_str() {
            "pedestrian" | "crossing" | "corridor" | "footway" | "path" => Road::Pedestrian,
            "cycleway" => Road::Bike,
            "residential" | "living_street" | "unclassified" | "service" | "track" => Road::Local,
            "secondary" | "secondary_link" | "tertiary" | "tertiary_link" | "none" => {
                Road::Collector
            }
            "primary" | "primary_link" => Road::Arterial,
            _ => {
                eprintln!("{}: Unexpected highway value: {}", self.way, self.highway);
                Road::Collector
            }
        }
    }

    /// Given these OSM tags, get the forward and reverse cycleways and directionality
    // opted to make this a mega function, since the logic for
    // determining these 3 was always coupled
    pub fn get_cycleways_and_directionality(&self) -> (Cycleway, Cycleway, Salmoning) {
        // easiest solution, leverage cycleway both if specified
        if let Some(cycleway) = self.get_cycleway_if_specified(&self.cycleway_both) {
            return (cycleway, cycleway, false);
        }

        // Leverage indicators for designated bike paths
        if self.highway == "cycleway" || self.bicycle == "designated" {
            return self.handle_designated_paths();
        }

        // Now handle oneway roads
        if self.oneway == "yes" {
            return self.handle_oneway_roads();
        }

        // finally, handle bidirectional roads
        self.handle_bidirectional_roads()
    }

    fn get_cycleway_from_tag(&self, val: &str) -> Cycleway {
        match val {
            "track" | "separate" => Cycleway::Track,
            "lane" | "shoulder" | "opposite_lane" => Cycleway::Lane,
            "shared_lane" | "share_busway" => Cycleway::Shared,
            "no" | "none" => Cycleway::No,
            _ => {
                eprintln!("{}: Unexpected cycleway value: {val}", self.way);
                Cycleway::No
            }
        }
    }

    fn get_cycleway_if_specified(&self, tag: &str) -> Option<Cycleway> {
        if tag != "none" {
            Some(self.get_cycleway_from_tag(tag))
        } else {
            None
        }
    }

    fn handle_designated_paths(&self) -> (Cycleway, Cycleway, Salmoning) {
        // Just need to check if this designated path is a oneway
        // A lack of these tags is an implicit _oneway=yes_
        if self.oneway == "no" || self.oneway_bicycle == "no" {
            (Cycleway::Track, Cycleway::Track, false)
        } else {
            (Cycleway::Track, Cycleway::Track, true)
        }
    }

    fn handle_oneway_roads(&self) -> (Cycleway, Cycleway, Salmoning) {
        // if right side is specified, use that
        if let Some(labels) = self.check_cycleway_side(
            &self.cycleway_right,
            &self.cycleway_right_oneway,
            &self.cycleway_left,
            &self.cycleway_left_oneway,
        ) {
            return labels;
        }

        // if left side is specified, use that
        if let Some(labels) = self.check_cycleway_side(
            &self.cycleway_left,
            &self.cycleway_left_oneway,
            &self.cycleway_right,
            &self.cycleway_right_oneway,
        ) {
            return labels;
        }

        // if we are this far down, there are no forward direction lanes
        // so begin checking for contraflow lanes
        if let Some(reverse_cycleway) =
            self.get_cycleway_if_contraflow(&self.cycleway_right_oneway, &self.cycleway_right)
        {
            return (Cycleway::No, reverse_cycleway, false);
        }
        if let Some(reverse_cycleway) =
            self.get_cycleway_if_contraflow(&self.cycleway_left_oneway, &self.cycleway_left)
        {
            return (Cycleway::No, reverse_cycleway, false);
        }

        // if there are no forward or backward bike lanes on this oneway road, default!
        (Cycleway::No, Cycleway::No, true)
    }

    /// For use with oneway roads: if there is non-contraflow bike infra on the specified side, use it
    /// Also check the opposite side for an explicit, different reverse lane
    fn check_cycleway_side(
        &self,
        cycleway_side: &str,
        cycleway_side_oneway: &str,
        opposite_side: &str,
        opposite_side_oneway: &str,
    ) -> Option<(Cycleway, Cycleway, Salmoning)> {
        if cycleway_side != "none" && cycleway_side != "no" && cycleway_side_oneway != "-1" {
            let cycleway = self.get_cycleway_from_tag(cycleway_side);

            // is this a bidirectional cycleway?
            let mut salmon = true;
            if cycleway_side_oneway == "no" || self.oneway_bicycle == "no" {
                salmon = false;
            }

            // does the opposite side have an explicit reverse lane?
            let mut reverse_cycleway = cycleway;
            if opposite_side_oneway == "-1" {
                reverse_cycleway = self.get_cycleway_from_tag(opposite_side);
                salmon = false;
            }

            return Some((cycleway, reverse_cycleway, salmon));
        }
        None
    }

    fn get_cycleway_if_contraflow(&self, oneway_tag: &str, cycleway_tag: &str) -> Option<Cycleway> {
        // https://wiki.openstreetmap.org/wiki/Key:cycleway:right:oneway
        if oneway_tag == "-1" {
            Some(self.get_cycleway_from_tag(cycleway_tag))
        } else {
            None
        }
    }

    fn handle_bidirectional_roads(&self) -> (Cycleway, Cycleway, Salmoning) {
        // first, check if either side uses bidirectional bike infra
        if let Some(cycleway) =
            self.get_cycleway_if_bidirectional(&self.cycleway_left, &self.cycleway_left_oneway)
        {
            return (cycleway, cycleway, false);
        }
        if let Some(cycleway) =
            self.get_cycleway_if_bidirectional(&self.cycleway_right, &self.cycleway_right_oneway)
        {
            return (cycleway, cycleway, false);
        }

        // otherwise, right side is the forward cycleway, left side is reverse cycleway
        let forward_cycleway = self.get_cycleway_from_tag(&self.cycleway_right);
        let reverse_cycleway = self.get_cycleway_from_tag(&self.cycleway_left);

        (forward_cycleway, reverse_cycleway, false)
    }

    fn get_cycleway_if_bidirectional(
        &self,
        cycleway_tag: &str,
        oneway_tag: &str,
    ) -> Option<Cycleway> {
        if oneway_tag == "no" {
            Some(self.get_cycleway_from_tag(cycleway_tag))
        } else {
            None
        }
    }
}
