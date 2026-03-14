/// Elevation lookup from GeoTIFF raster data.
/// Used at ETL time to compute per-segment elevation gain/loss.
#[cfg(feature = "elevation")]
use gdal::Dataset;

#[cfg(feature = "elevation")]
pub struct ElevationLookup {
    dataset: Dataset,
    /// GeoTransform: [origin_x, pixel_width, 0, origin_y, 0, pixel_height]
    transform: [f64; 6],
    /// Inverse transform for coordinate → pixel mapping
    inv_transform: [f64; 6],
    x_size: usize,
    y_size: usize,
}

#[cfg(feature = "elevation")]
impl ElevationLookup {
    pub fn new(path: &str) -> Result<Self, anyhow::Error> {
        let dataset = Dataset::open(path)?;
        let transform = dataset.geo_transform()?;

        // Compute inverse transform for lon/lat → pixel coords
        let det = transform[1] * transform[5] - transform[2] * transform[4];
        let inv_transform = [
            (transform[2] * transform[3] - transform[0] * transform[5]) / det,
            transform[5] / det,
            -transform[2] / det,
            (transform[0] * transform[4] - transform[1] * transform[3]) / det,
            -transform[4] / det,
            transform[1] / det,
        ];

        let (x_size, y_size) = dataset.raster_size();

        eprintln!(
            "Elevation raster loaded: {}x{}, origin=({}, {})",
            x_size, y_size, transform[0], transform[3]
        );

        Ok(Self {
            dataset,
            transform,
            inv_transform,
            x_size,
            y_size,
        })
    }

    /// Sample elevation at a single (lon, lat) point.
    /// Returns elevation in meters, or None if outside raster or nodata.
    pub fn get_elevation(&self, lon: f64, lat: f64) -> Option<f32> {
        // Convert lon/lat to pixel coordinates using inverse geotransform
        let px = self.inv_transform[0] + self.inv_transform[1] * lon + self.inv_transform[2] * lat;
        let py = self.inv_transform[3] + self.inv_transform[4] * lon + self.inv_transform[5] * lat;

        let col = px.floor() as isize;
        let row = py.floor() as isize;

        if col < 0 || row < 0 || col >= self.x_size as isize || row >= self.y_size as isize {
            return None;
        }

        let band = self.dataset.rasterband(1).ok()?;
        let buf = band
            .read_as::<f32>((col as isize, row as isize), (1, 1), (1, 1), None)
            .ok()?;

        let value = buf.data()[0];

        // Check for nodata
        if let Some(nodata) = band.no_data_value() {
            if (value as f64 - nodata).abs() < 0.01 {
                return None;
            }
        }

        Some(value)
    }

    /// Compute elevation gain and loss along a straight-line segment by sampling
    /// at regular intervals. Returns (gain, loss) in meters as i16.
    ///
    /// Sampling strategy:
    /// - distance <= 10m: endpoints only
    /// - distance > 10m: every ~10m, always including endpoints + at least 1 midpoint
    pub fn compute_segment_elevation(
        &self,
        start_lon: f64,
        start_lat: f64,
        end_lon: f64,
        end_lat: f64,
        distance_m: i32,
    ) -> (i16, i16) {
        let num_samples = if distance_m <= 10 {
            2 // endpoints only
        } else {
            // ceil(distance/10) + 1 ensures we get endpoints + at least 1 midpoint
            ((distance_m as f64 / 10.0).ceil() as usize + 1).max(3)
        };

        // Sample elevations at evenly-spaced points along the segment
        let mut elevations: Vec<f32> = Vec::with_capacity(num_samples);
        for i in 0..num_samples {
            let t = i as f64 / (num_samples - 1) as f64;
            let lon = start_lon + t * (end_lon - start_lon);
            let lat = start_lat + t * (end_lat - start_lat);

            if let Some(elev) = self.get_elevation(lon, lat) {
                elevations.push(elev);
            }
            // Skip nodata points — they won't contribute to gain/loss
        }

        if elevations.len() < 2 {
            // Sentinel: -1 signals "no elevation data available" so the cost
            // function can penalize these segments above the flat baseline
            return (-1, -1);
        }

        // Walk the sampled elevations, summing positive and negative deltas
        let mut gain: f32 = 0.0;
        let mut loss: f32 = 0.0;
        for window in elevations.windows(2) {
            let delta = window[1] - window[0];
            if delta > 0.0 {
                gain += delta;
            } else {
                loss += -delta;
            }
        }

        (gain.round() as i16, loss.round() as i16)
    }
}
