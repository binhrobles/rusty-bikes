use flate2::{
    write::{GzEncoder, ZlibEncoder},
    Compression,
};
use std::{io::prelude::*, time::Instant};
use tracing::debug;

type CompressionOutput = Vec<u8>;

#[derive(Default, Debug, PartialEq)]
pub enum Encoding {
    Gzip,
    Zlib,

    #[default]
    No,
}

impl TryFrom<&str> for Encoding {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.contains(&Encoding::Gzip.to_string()) {
            return Ok(Encoding::Gzip);
        }
        if value.contains(&Encoding::Zlib.to_string()) {
            return Ok(Encoding::Zlib);
        }

        Ok(Encoding::No)
    }
}

impl ToString for Encoding {
    fn to_string(&self) -> String {
        match self {
            Encoding::Gzip => "gzip".to_string(),
            Encoding::Zlib => "deflate".to_string(),
            Encoding::No => "".to_string(),
        }
    }
}

pub fn compress_with_encoding(
    body: &str,
    accept_encoding: &str,
) -> Result<(Option<CompressionOutput>, Encoding), anyhow::Error> {
    let now = Instant::now();
    match accept_encoding.try_into() {
        Ok(Encoding::Gzip) => {
            let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(body.as_bytes())?;
            let byte_array = encoder.finish()?;

            debug!(
                raw = body.len() / 1024,
                compressed = byte_array.len() / 1024,
                time = now.elapsed().as_millis()
            );

            Ok((Some(byte_array), Encoding::Gzip))
        }
        Ok(Encoding::Zlib) => {
            let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(body.as_bytes())?;
            let byte_array = encoder.finish()?;

            debug!(
                raw = body.len() / 1024,
                compressed = byte_array.len() / 1024,
                time = now.elapsed().as_millis()
            );

            Ok((Some(byte_array), Encoding::Zlib))
        }
        _ => {
            debug!("No compression");
            Ok((None, Encoding::No))
        }
    }
}
