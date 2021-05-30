use cgmath::{InnerSpace, Matrix4, Point2, Vector2, Zero};
use std::time::Duration;

use super::{Scale, TilePos};

/// Minimum target width & height, to avoid divide-by-zero errors.
const MIN_TARGET_SIZE: u32 = 10;

/// Number of pixels to pan that feels equivalent to scaling by a factor of 2.
///
/// Pixels are a very small unit compared to logarithmic scale factor, and
/// panning 400 pixels feels about equivalent to scaling by a factor of 2 to me.
///
/// Obviously this depends on DPI and/or window size, but deriving an absolute
/// formula for it is a nightmare of calculus. All that matters is it's vaguely
/// proportional to the size of the window, so at some point in the future this
/// could be changed to something like sqrt(h²+w²) / 5. Here's a Desmos link if
/// you're curious: https://www.desmos.com/calculator/1yxv7mglnj.
pub(super) const PIXELS_PER_2X_SCALE: f64 = 400.0;

/// Distance beneath which to "snap" to the target, for interpolation strategies
/// like exponential decay that never actually reach their target.
const INTERPOLATION_DISTANCE_THRESHOLD: f64 = 0.001;
/// Exponential decay constant used for interpolation.
const INTERPOLATION_DECAY_CONSTANT: f64 = 0.04;

/// 2D camera.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Camera {
    /// Width and height of the render target.
    target_dimensions: (u32, u32),
    /// Display scaling factor.
    dpi: f32,

    /// Tile coordinates at the center of the camera.
    center: Point2<f64>,
    /// Scale factor.
    scale: Scale,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            target_dimensions: (MIN_TARGET_SIZE, MIN_TARGET_SIZE),
            dpi: 1.0,

            center: Point2::new(0.0, 0.0),
            scale: Scale::default(),
        }
    }
}

impl Camera {
    /// Returns a new camera at the center of the grid.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the width and height of the render target.
    pub fn target_dimensions(self) -> (u32, u32) {
        self.target_dimensions
    }
    /// Sets the width and height of the render target.
    pub fn set_target_dimensions(&mut self, (target_w, target_h): (u32, u32)) {
        self.target_dimensions = (
            std::cmp::max(MIN_TARGET_SIZE, target_w),
            std::cmp::max(MIN_TARGET_SIZE, target_h),
        );
    }
    /// Returns the display scaling factor, which does not affect rendering of
    /// tiles but may affect other UI elements.
    pub fn dpi(self) -> f32 {
        self.dpi
    }
    /// Sets the display scaling factor.
    pub fn set_dpi(&mut self, dpi: f32) {
        self.dpi = dpi;
    }

    /// Returns the position of the center of the camera.
    pub fn center(self) -> Point2<f64> {
        self.center
    }
    /// Sets the position of the center of the camera.
    pub fn set_center(&mut self, pos: Point2<f64>) {
        self.center = pos;
    }

    /// Returns the visual scale of tiles.
    pub fn scale(self) -> Scale {
        self.scale
    }
    /// Sets the visual scale of tiles.
    pub fn set_scale(&mut self, scale: Scale) {
        self.scale = scale.clamp();
    }

    /// Sets the visual scale of tiles, keeping one point at the same location
    /// on the screen.
    ///
    /// If `invariant_pos` is `None`, then the value returned by `center()` is
    /// used instead.
    pub fn scale_to(&mut self, scale: Scale, invariant_pos: Option<Point2<f64>>) {
        // Scale, keeping the center position invariant.
        let old_scale = self.scale();
        self.set_scale(scale);
        let new_scale = self.scale(); // `.clamp()`ed in `set_scale()`

        // Compute tile offset of `invariant_pos` from the center.
        let invariant_pos_offset = invariant_pos
            .map(|invar_pos| invar_pos - self.center())
            .unwrap_or_else(Vector2::zero);

        // Compute the old scaled offset of `invariant_pos` from the center.
        let invariant_pos_old_scaled_offset = invariant_pos_offset * old_scale.factor();
        // Compute the new scaled offset of `invariant_pos` from the center.
        let invariant_pos_new_scaled_offset = invariant_pos_offset * new_scale.factor();

        // Compute the difference between those scaled offsets.
        let delta_pixel_offset = invariant_pos_new_scaled_offset - invariant_pos_old_scaled_offset;

        // Apply that offset so that the point goes back to the same scaled
        // location as before.
        self.set_center(self.center() + delta_pixel_offset / new_scale.factor());
    }
    /// Scales by 2^`log2_factor`, keeping one invariant point at the same
    /// location on the screen.
    ///
    /// If `invariant_pos` is `None`, then the value returned by `pos()` is
    /// invariant.
    pub fn scale_by_log2_factor(&mut self, log2_factor: f64, invariant_pos: Option<Point2<f64>>) {
        self.scale_to(
            Scale::from_log2_factor(self.scale().log2_factor() + log2_factor),
            invariant_pos,
        );
    }
    /// Scales by the given factor, keeping one invariant point at the same
    /// location on the screen.
    ///
    /// If `invariant_pos` is `None`, then the value returned by `pos()` is
    /// invariant.
    pub fn scale_by_factor(&mut self, factor: f64, invariant_pos: Option<Point2<f64>>) {
        assert!(
            factor > 0.0,
            "Scale factor must be a positive number, not {}",
            factor,
        );
        self.scale_by_log2_factor(factor.log2(), invariant_pos)
    }
    /// Snaps to the nearest power-of-2 scale factor, keeping one invariant
    /// point at the same location on the screen.
    ///
    /// If `invariant_pos` is `None`, then the value returned by `pos()` is
    /// invariant.
    pub fn snap_scale(&mut self, invariant_pos: Option<Point2<f64>>) {
        self.scale_by_factor(self.scale().round() / self.scale(), invariant_pos);
        self.set_scale(self.scale().round()); // Fix any potential rounding error.
    }

    /// Returns the abstract "distance" between two cameras.
    pub fn distance(a: Self, b: Self) -> f64 {
        let avg_scale = average_lerped_scale(a.scale(), b.scale());
        let total_tiles_delta = (b.center() - a.center()).magnitude();
        let total_pixels_delta = total_tiles_delta * avg_scale.factor();
        // Divide by a constant factor to bring translation and scale into the
        // same arbitrary units of optical flow.
        let panning_distance = total_pixels_delta / PIXELS_PER_2X_SCALE;
        let scale_distance = a.scale().log2_factor() - b.scale().log2_factor();
        // Use euclidean distance.
        let squared_panning_distance = panning_distance * panning_distance;
        let squared_scale_distance = scale_distance * scale_distance;
        let squared_distance = squared_panning_distance + squared_scale_distance;
        squared_distance.sqrt()
    }

    /// Returns a camera that is some fraction 0.0 <= t <= 1.0 of the distance
    /// between two cameras using linear interpolation that preserves the
    /// fixed point of the transformation from one camera to the other.
    ///
    /// This function attempts to interpolate linearly with respect to the
    /// apparent motion experienced by the user, so it linearly interpolates 2D
    /// panning speed in terms of on-screen pixels rather than tiles, and
    /// interpolates scale factor logarithmically.
    #[must_use = "This method returns a new value instead of mutating its input"]
    fn lerp(a: Self, b: Self, t: f64) -> Self {
        let mut ret = a.clone();

        // When interpolating position and scale together, we would want the
        // following constraints:
        //
        // 1. Finish scaling and panning at the same time.
        // 2. Keep scaling "speed" consistent -- scale by the same factor each
        //    frame by using lerping the logarithm of the scale factor.
        // 3. Keep panning "speed" consistent -- pan by the same number of
        //    PIXELS each frame (not necessarilly the same number of TILES).
        //
        // All of these together have the nice property of maintaining the fixed
        // point of the transformation throughout the transformation. (See
        // https://www.youtube.com/watch?v=csInNn6pfT4 for more on fixed
        // points.) Scaling using the scroll wheel uses the mouse position on
        // the grid as a fixed point, so this point stays still, which gives a
        // smooth experience for the user.
        //
        // #1 is trivial -- just start both transformations at t=0 and end them
        // both at t=1. The hard part is finding the difference in pixels, and
        // panning that many pixels (integrated over the change in scale).

        // Interpolate scale factor logarithmically.
        let delta_scale_factor = b.scale / a.scale;
        ret.scale_by_factor(delta_scale_factor.powf(t), None);

        // Read the comments in `average_lerped_scale()` before proceeding.
        let avg_scale = average_lerped_scale(a.scale, b.scale);
        // Convert the tile distance to the total number of pixels to travel.
        let total_pixels_delta = (b.center - a.center) * avg_scale.factor();

        // Now that we know the number of pixels to travel in the whole timestep
        // of 0 <= t <= 1, we have to figure out how many tiles to travel during
        // 0 <= t <= T, where T is the "destination" time (argument to this
        // function). We can compute the average scale of this smaller
        // interpolation just ranging from 0 to T using `average_lerped_scale`
        // again, but using s(T) instead of s₂.
        let zt = average_lerped_scale(a.scale, ret.scale);
        // Multiply the total number of pixels to travel by T to get the number
        // of pixels to travel on 0 <= t <= T.
        let pixels_delta = total_pixels_delta * t;
        // Finally, divide by the new scale factor to get the number of tiles to
        // travel on 0 <= t <= T.
        let tiles_delta = pixels_delta / zt.factor();
        ret.center += tiles_delta;

        ret
    }
    /// Advances the camera by one frame toward another camera.
    ///
    /// Returns `true` if the target has been reached, or `false` otherwise.
    pub fn advance_interpolation(&mut self, target: Self, frame_duration: Duration) -> bool {
        if *self == target {
            true
        } else if Self::distance(*self, target) < INTERPOLATION_DISTANCE_THRESHOLD {
            *self = target;
            true
        } else {
            let t = frame_duration.as_secs_f64() / INTERPOLATION_DECAY_CONSTANT;
            *self = Self::lerp(
                *self,
                target,
                // Clamp to 0 <= t <= 1. `min()` comes first so that `NaN`s
                // will become `1.0`.
                t.min(1.0).max(0.0),
            );
            false
        }
    }

    /// Returns an integer tile position near the center of the camera.
    pub fn int_center(self) -> [i32; 2] {
        [self.center.x as i32, self.center.y as i32]
    }

    /// Returns the tile transform matrix relative to `int_center()`.
    pub fn gl_matrix(self) -> Matrix4<f32> {
        let [int_x, int_y] = self.int_center();
        let int_center_f64 = Point2::new(int_x as f64, int_y as f64);
        let mut displacement = -(self.center - int_center_f64);
        if self.scale.log2_factor().fract().is_zero() {
            // When the scale factor is an exact power of two, round to the
            // nearest pixel to make the final image more crisp. This is
            // disabled otherwise because it causes noticeable jiggling during
            // interpolation.
            let mut pixel_displacement = displacement * self.scale.factor();
            pixel_displacement.x = pixel_displacement.x.round();
            pixel_displacement.y = pixel_displacement.y.round();
            // Offset by half a pixel if the target dimensions are odd, so that
            // tile boundaries line up with pixel boundaries.
            let (target_w, target_h) = self.target_dimensions();
            if target_w % 2 == 1 {
                pixel_displacement.x += 0.5_f64;
            }
            if target_h % 2 == 1 {
                pixel_displacement.y += 0.5_f64;
            }
            displacement = pixel_displacement / self.scale.factor();
        }

        let scale_matrix = cgmath::Matrix4::from_scale(self.scale.factor());
        let translate_matrix = cgmath::Matrix4::from_translation(displacement.extend(0.0));
        let tile_transform_matrix = (scale_matrix * translate_matrix).cast().unwrap();

        self.projection_matrix() * tile_transform_matrix
    }

    /// Returns the orthographic projection matrix based on the target
    /// dimensions.
    fn projection_matrix(self) -> cgmath::Matrix4<f32> {
        let (target_w, target_h) = self.target_dimensions;
        let sx = 2.0 / target_w as f32;
        let sy = 2.0 / target_h as f32;
        let sz = 1.0;
        cgmath::Matrix4::from_nonuniform_scale(sx, sy, sz)
    }

    /// Returns the global tile coordinates of a pixel.
    pub fn pixel_to_tile_coords(self, (x, y): (u32, u32)) -> Point2<f64> {
        let (target_w, target_h) = self.target_dimensions;
        let x = x as f64 - target_w as f64 / 2.0;
        let y = -(y as f64 - target_h as f64 / 2.0);

        Point2::new(
            x / self.scale.factor() + self.center.x,
            y / self.scale.factor() + self.center.y,
        )
    }
    /// Returns the global integer coordinates of the tile containing a pixel.
    pub fn pixel_to_tile_pos(self, pixel: (u32, u32)) -> TilePos {
        let t = self.pixel_to_tile_coords(pixel);
        TilePos(t.x.floor() as i32, t.y.floor() as i32)
    }
}

/// Returns the "average" scale between the two cameras, averaging scale factor
/// linearly with respect to time during a linear interpolation, where scale
/// factor is interpolated logarithmically.
///
/// Read source comments to see how this is relevant.
fn average_lerped_scale(s1: Scale, s2: Scale) -> Scale {
    // Read the comments in the first half of `Camera2D::lerp()` before
    // proceeding.
    //
    // We want to find the total number of pixels to travel. The logarithm of
    // the scale factor is a linear function of time s(t) = s₁ + (s₂ - s₁) * t
    // for 0 <= t <= 1, where s₁ and s₂ are the logarithms of the inital and
    // final scales respectively. The number of pixels to travel is a constant
    // value for that range as well. The number of tiles per pixel is
    // 1/(2^s(t)), so the total number of tiles to travel is the integral of
    // pixels/(2^s(t)) dt from t=0 to t=1. That integral comes out to the
    // following:
    //
    //           pixels * ( 2^(-s₁) - 2^(-s₂) )
    // tiles = - ------------------------------
    //                 ln(2) * (s₁ - s₂)
    //
    // (Note the negative sign in front.) We know how many tiles to travel;
    // that's just b.pos - a.pos. We could solve the above equation for the
    // number of pixels, but instead let's solve it for the ratio of pixels to
    // tiles; i.e. the average scale factor:
    //
    // pixels     ln(2) * (s₁ - s₂)
    // ------ = - -----------------
    // tiles      2^(-s₁) - 2^(-s₂)

    let numerator = 2.0_f64.ln() * (s1.log2_factor() - s2.log2_factor());
    let denominator = s1.inv_factor() - s2.inv_factor();
    if numerator.is_zero() || denominator.is_zero() {
        // The expression is undefined at s₁ = s₂, but then the average of s₁
        // and s₂ is trivial.
        s1
    } else {
        Scale::from_factor(-numerator / denominator)
    }
}
