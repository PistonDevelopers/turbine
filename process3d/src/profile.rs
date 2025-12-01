//! Performance profiling.

use crate::mask::CompressedMasks;
use crate::PixelPos;

/// Data that is sent to performance profiler after pre-processing.
pub struct ProfileCompressData<'a> {
    /// Tile size.
    pub tile_size: u32,
    /// The size of render tile grid.
    pub grid: PixelPos,
    /// Compressed masks per render tile.
    pub compr_masks: &'a [CompressedMasks],
}

/// Returns the amount of seconds since UNIX EPOCH.
pub fn now() -> f64 {
    use std::time::{SystemTime, UNIX_EPOCH};

    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(val) => val.as_secs() as f64 +
                   f64::from(val.subsec_nanos()) / 1.0e9,
        Err(err) => -{
            let val = err.duration();
            val.as_secs() as f64 +
            f64::from(val.subsec_nanos()) / 1.0e9
        }
    }
}
