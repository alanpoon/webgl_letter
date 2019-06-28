use std::time::Duration;

use crate::math::prelude::{MetricSpace, Vector2};
use crate::utils::hash::{FastHashMap, FastHashSet};
use crate::utils::time::Timestamp;

/// The setup parameters of mouse device.
///
/// Notes that the `distance` series paramters are measured in points.
#[derive(Debug, Clone, Copy)]
pub struct TextEditParams {

}

impl Default for TextEditParams {
    fn default() -> Self {
        TextEditParams {
        }
    }
}

pub struct TextEdit {
    params: TextEditParams,
    downs: FastHashSet<(String,String)>,
}

impl TextEdit {
    pub fn new(params: TextEditParams) -> Self {
        TextEdit {
            params,
            downs: FastHashSet::default(),
        }
    }
    #[inline]
    pub fn advance(&mut self) {
        self.downs.clear();
    }
    #[inline]
    pub fn reset(&mut self) {
        self.downs.clear();
    }

    #[inline]
    pub fn on(&mut self, down: (String, String)) {
        if !self.downs.contains(&down) {
            self.downs.insert(down);
        }
    }

    #[inline]
    pub fn downs(&self) -> FastHashSet<(String,String)> {
        self.downs.clone()
    }
}
