use std::fmt;
use std::ops::*;

/// Scale factor.
///
/// This is the width or height of a square tile in pixels.
///
/// When zoomed in close, the scale factor is large, and the base-2 logarithm of
/// the scale factor is positive.
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct Scale {
    log2_factor: f64,
}
impl Default for Scale {
    fn default() -> Self {
        // 16 pixels per tile is the default in classic Minesweeper.
        Self::from_factor(16.0)
    }
}
impl fmt::Display for Scale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:1", self.log2_factor.exp2().round())
    }
}

impl Scale {
    /// The lower scale limit; i.e. the furthest the player can zoom out.
    const LOWER_LIMIT: f64 = 3.0;
    /// The upper scale limit; i.e. the furthest the player can zoom in.
    const UPPER_LIMIT: f64 = 6.0;

    /// Creates a `Scale` from a scale factor's base-2 logarithm (e.g. `3.0` = 8:1 scale).
    pub fn from_log2_factor(log2_factor: f64) -> Self {
        Self { log2_factor }
    }
    /// Creates a `Scale` from a scale factor (e.g. `8` = 8:1 scale).
    ///
    /// # Panics
    ///
    /// This function panics if `factor` is not greater than zero.
    pub fn from_factor(factor: f64) -> Self {
        Self {
            log2_factor: factor.log2(),
        }
    }

    /// Clamps the scale to the lower and upper limits. This is not
    /// automatically enforced by `Scale`; it must be called manually.
    #[must_use = "This method returns a new value instead of mutating its input"]
    pub fn clamp(self) -> Self {
        if self.log2_factor < Self::LOWER_LIMIT {
            Self::from_log2_factor(Self::LOWER_LIMIT)
        } else if self.log2_factor > Self::UPPER_LIMIT {
            Self::from_log2_factor(Self::UPPER_LIMIT)
        } else {
            self
        }
    }

    /// Returns the base-2 logarithm of the scale factor (e.g. -2.0 = 4:1 scale).
    pub fn log2_factor(self) -> f64 {
        self.log2_factor
    }
    /// Returns the scale factor, which is the length of pixels per tile.
    pub fn factor(self) -> f64 {
        self.log2_factor().exp2()
    }
    /// Returns the inverse scale factor.
    pub fn inv_factor(self) -> f64 {
        1.0 / self.factor()
    }

    /// Rounds the scale factor to the nearest power of 2.
    pub fn round(self) -> Self {
        Self {
            log2_factor: self.log2_factor.round(),
        }
    }
    /// Rounds the scale factor down (zooms out) to the nearest power of 2.
    pub fn floor(self) -> Self {
        Self {
            log2_factor: self.log2_factor.floor(),
        }
    }
    /// Rounds the scale factor up (zooms in) to the nearest power of 2.
    pub fn ceil(self) -> Self {
        Self {
            log2_factor: self.log2_factor.ceil(),
        }
    }
}

impl Mul<f64> for Scale {
    type Output = Self;

    /// Scales up / zooms in by a factor.
    fn mul(self, factor: f64) -> Self {
        Self::from_log2_factor(self.log2_factor + factor.log2())
    }
}
impl MulAssign<f64> for Scale {
    /// Scales up / zooms in by a factor.
    fn mul_assign(&mut self, factor: f64) {
        self.log2_factor += factor.log2();
    }
}

impl Div<f64> for Scale {
    type Output = Self;

    /// Scales down / zooms out by a factor.
    fn div(self, factor: f64) -> Self {
        Self::from_log2_factor(self.log2_factor - factor.log2())
    }
}
impl DivAssign<f64> for Scale {
    /// Scales down / zooms out by a factor.
    fn div_assign(&mut self, factor: f64) {
        self.log2_factor -= factor.log2();
    }
}

impl Div<Scale> for Scale {
    type Output = f64;

    /// Computes the ratio between two scales.
    ///
    /// # Panics
    ///
    /// This operation panics if the result does not fit in an `f64`.
    fn div(self, other: Self) -> f64 {
        (self.log2_factor - other.log2_factor).exp2()
    }
}
