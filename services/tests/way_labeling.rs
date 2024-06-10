use rusty_router::{graph::Graph, osm::{Cycleway, WayLabels}, osm::{Road, WayId}};

fn assert_helper(expected: WayLabels, actual: WayLabels, header: WayId) {
    assert!(expected == actual,
        "{header} | expected: {:?}, actual: {:?}",
        expected, actual)
}

#[test]
fn bidirectional_road_w_one_bike_lane() -> Result<(), anyhow::Error> {
    let graph = Graph::new()?;
    let way = 654744285;

    let labels = graph.get_way_labels(way)?; // https://www.openstreetmap.org/way/654744285
    assert_helper((Cycleway::Lane, Road::Local, false), labels, way);

    let labels = graph.get_way_labels(-way)?; // https://www.openstreetmap.org/way/654744285
    assert_helper((Cycleway::Shared, Road::Local, false), labels, -way);

    Ok(())
}

#[test]
fn bidirectional_road_w_two_different_bike_lanes() -> Result<(), anyhow::Error> {
    let graph = Graph::new()?;
    let way = 494221659;

    let labels = graph.get_way_labels(way)?;
    assert_helper((Cycleway::Shared, Road::Collector, false), labels, way);

    let labels = graph.get_way_labels(-way)?;
    assert_helper((Cycleway::Lane, Road::Collector, false), labels, -way);

    Ok(())
}

#[test]
fn bidirectional_road_w_bidirectional_bike_lane() -> Result<(), anyhow::Error> {
    let graph = Graph::new()?;
    let way = 464964299;

    let labels = graph.get_way_labels(way)?;
    assert_helper((Cycleway::Track, Road::Collector, false), labels, way);

    let labels = graph.get_way_labels(-way)?;
    assert_helper((Cycleway::Track, Road::Collector, false), labels, -way);

    Ok(())
}

#[test]
fn bidirectional_road_w_shared_bike_lane_both_ways() -> Result<(), anyhow::Error> {
    let graph = Graph::new()?;
    let way = 68523765;

    let labels = graph.get_way_labels(way)?;
    assert_helper((Cycleway::Shared, Road::Collector, false), labels, way);

    let labels = graph.get_way_labels(-way)?;
    assert_helper((Cycleway::Shared, Road::Collector, false), labels, -way);

    Ok(())
}

#[test]
fn bidirectional_arterial_w_no_bike_infra() -> Result<(), anyhow::Error> {
    let graph = Graph::new()?;
    let way = 420880039;

    let labels = graph.get_way_labels(way)?;
    assert_helper((Cycleway::Shared, Road::Arterial, false), labels, way);

    let labels = graph.get_way_labels(-way)?;
    assert_helper((Cycleway::Shared, Road::Arterial, false), labels, -way);

    Ok(())
}

#[test]
fn one_way_road_w_left_side_bike_lane() -> Result<(), anyhow::Error> {
    let graph = Graph::new()?;
    let way = 844446016;

    let labels = graph.get_way_labels(way)?;
    assert_helper((Cycleway::Lane, Road::Local, false), labels, way);

    let labels = graph.get_way_labels(-way)?;
    assert_helper((Cycleway::Lane, Road::Local, true), labels, -way);

    Ok(())
}

#[test]
fn one_way_road_w_right_side_bike_lane() -> Result<(), anyhow::Error> {
    let graph = Graph::new()?;
    let way = 420572575;

    let labels = graph.get_way_labels(way)?;
    assert_helper((Cycleway::Track, Road::Collector, false), labels, way);

    let labels = graph.get_way_labels(-way)?;
    assert_helper((Cycleway::Track, Road::Collector, true), labels, -way);

    Ok(())
}

#[test]
fn one_way_road_w_bidirectional_bike_lane() -> Result<(), anyhow::Error> {
    let graph = Graph::new()?;
    let way = 1031982495;

    let labels = graph.get_way_labels(way)?;
    assert_helper((Cycleway::Track, Road::Collector, false), labels, way);

    let labels = graph.get_way_labels(-way)?;
    assert_helper((Cycleway::Track, Road::Collector, false), labels, -way);

    Ok(())
}

#[test]
fn one_way_road_w_single_contraflow_bike_lane() -> Result<(), anyhow::Error> {
    let graph = Graph::new()?;
    let way = 455014439;

    let labels = graph.get_way_labels(way)?;
    assert_helper((Cycleway::Shared, Road::Local, false), labels, way);

    let labels = graph.get_way_labels(-way)?;
    assert_helper((Cycleway::Track, Road::Local, false), labels, -way);

    Ok(())
}

#[test]
fn one_way_road_w_contraflow_bidirectional_bike_lane() -> Result<(), anyhow::Error> {
    let graph = Graph::new()?;
    let way = 1258745670;

    let labels = graph.get_way_labels(way)?;
    assert_helper((Cycleway::Shared, Road::Local, false), labels, way);

    let labels = graph.get_way_labels(-way)?;
    assert_helper((Cycleway::Lane, Road::Local, false), labels, -way);

    Ok(())
}

#[test]
fn bidirectional_bike_lane() -> Result<(), anyhow::Error> {
    let graph = Graph::new()?;
    let way = 505864686;

    let labels = graph.get_way_labels(way)?;
    assert_helper((Cycleway::Track, Road::Bike, false), labels, way);

    let labels = graph.get_way_labels(-way)?;
    assert_helper((Cycleway::Track, Road::Bike, false), labels, -way);

    Ok(())
}

#[test]
fn oneway_bike_lane() -> Result<(), anyhow::Error> {
    let graph = Graph::new()?;
    let way = 1232753103;

    let labels = graph.get_way_labels(way)?;
    assert_helper((Cycleway::Track, Road::Bike, false), labels, way);

    let labels = graph.get_way_labels(-way)?;
    assert_helper((Cycleway::Track, Road::Bike, true), labels, -way);

    Ok(())
}
