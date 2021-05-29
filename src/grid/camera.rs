use cgmath::{InnerSpace, Matrix4, Point2, SquareMatrix, Vector2, Zero};

use super::Scale;

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
const PIXELS_PER_2X_SCALE: f64 = 400.0;

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
    /// Returns the width and height of the render target.
    fn target_dimensions(self) -> (u32, u32) {
        self.target_dimensions
    }
    /// Sets the width and height of the render target.
    fn set_target_dimensions(&mut self, (target_w, target_h): (u32, u32)) {
        self.target_dimensions = (
            std::cmp::max(MIN_TARGET_SIZE, target_w),
            std::cmp::max(MIN_TARGET_SIZE, target_h),
        );
    }
    /// Returns the display scaling factor, which does not affect rendering of
    /// tiles but may affect other UI elements.
    fn dpi(self) -> f32 {
        self.dpi
    }
    /// Sets the display scaling factor.
    fn set_dpi(&mut self, dpi: f32) {
        self.dpi = dpi;
    }

    /// Returns the position of the center of the camera.
    fn center(self) -> Point2<f64> {
        self.center
    }
    /// Sets the position of the center of the camera.
    fn set_center(&mut self, pos: Point2<f64>) {
        self.center = pos;
    }

    /// Returns the visual scale of tiles.
    fn scale(self) -> Scale {
        self.scale
    }
    /// Sets the visual scale of tiles.
    fn set_scale(&mut self, scale: Scale) {
        self.scale = scale.clamp();
    }

    /// Sets the visual scale of tiles, keeping one point at the same location
    /// on the screen.
    ///
    /// If `invariant_pos` is `None`, then the value returned by `center()` is
    /// used instead.
    fn scale_to(&mut self, scale: Scale, invariant_pos: Option<Point2<f64>>) {
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
    fn scale_by_log2_factor(&mut self, log2_factor: f64, invariant_pos: Option<Point2<f64>>) {
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
    fn scale_by_factor(&mut self, factor: f64, invariant_pos: Option<Point2<f64>>) {
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
    fn snap_scale(&mut self, invariant_pos: Option<Point2<f64>>) {
        self.scale_by_factor(self.scale().round() / self.scale(), invariant_pos);
        self.set_scale(self.scale().round()); // Fix any potential rounding error.
    }

    /// Returns the abstract "distance" between two cameras.
    fn distance(a: Self, b: Self) -> f64 {
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

    // /// Returns the tile transform for this camera.
    // fn tile_transform(self) -> NdTileTransform<D> {
    //     let (render_tile_layer, render_tile_scale) = self.render_tile_layer_and_scale();
    //     let camera_transform = Matrix4::identity();
    //     let camera_center = if render_tile_scale.log2_factor().fract().is_zero() {
    //         // When the scale factor is an exact power of two, round to the
    //         // nearest pixel to make the final image more crisp. This is
    //         // disabled otherwise because it causes noticeable jiggling during
    //         // interpolation.
    //         let mut center_in_pixel_units = self.scale.tiles_to_units(self.center);
    //         center_in_pixel_units = center_in_pixel_units.round().to_fixedvec();
    //         // Offset by half a pixel if the target dimensions are odd, so that
    //         // tiles boundaries line up with pixel boundaries.
    //         let (target_w, target_h) = self.target_dimensions();
    //         if target_w % 2 == 1 {
    //             center_in_pixel_units[X] += 0.5_f64;
    //         }
    //         if target_h % 2 == 1 {
    //             center_in_pixel_units[Y] += 0.5_f64;
    //         }
    //         self.scale.units_to_tiles(center_in_pixel_units)
    //     } else {
    //         self.center.clone()
    //     };

    //     TileTransform2D::new(
    //         camera_center,
    //         render_tile_layer,
    //         render_tile_scale,
    //         camera_transform,
    //         ProjectionType::Orthographic,
    //         self.target_dimensions(),
    //     )
    // }

    // /// Returns a rectangle of tiles that are at least partially visible,
    // /// rounded outward to the nearest render tile.
    // fn global_visible_rect(self) -> BigRect<D> {
    //     // Compute the width and height of individual tiles that fit on the
    //     // screen.
    //     let (target_w, target_h) = self.target_dimensions();
    //     let target_pixels_size: IVec2D = NdVec([target_w as isize, target_h as isize]);
    //     let target_tiles_size: FixedVec2D = self
    //         .scale()
    //         .units_to_tiles(target_pixels_size.to_fixedvec());
    //     // Compute the tile vector pointing from the center of the screen to the
    //     // top right corner; i.e. the "half diagonal."
    //     let half_diag: FixedVec2D = target_tiles_size / 2.0;

    //     // Round to render tile boundaries.
    //     let render_tile_layer = self.render_tile_layer();
    //     render_tile_layer.round_rect(&BigRect2D::centered(
    //         self.center().floor(),
    //         &half_diag.ceil(),
    //     ))
    // }

    // /// Returns a drag update function for `DragViewCmd::Pan`.
    // fn drag_pan(self, cursor_start: FVec2D) -> Option<DragUpdateViewFn<Self>> {
    //     let start = self.tile_transform().pixel_to_global_pos(cursor_start);
    //     Some(Box::new(move |this, cursor_end| {
    //         let end = this.tile_transform().pixel_to_global_pos(cursor_end);
    //         this.center += start.clone() - end;
    //         Ok(DragOutcome::Continue)
    //     }))
    // }

    // /// Returns a drag update function for `DragViewCmd::Scale`.
    // fn drag_scale(self, cursor_start: FVec2D) -> Option<DragUpdateViewFn<Self>> {
    //     let initial_scale = self.scale();
    //     Some(Box::new(move |this, cursor_end| {
    //         let delta =
    //             (cursor_end - cursor_start)[Axis::Y] / -CONFIG.lock().ctrl.pixels_per_2x_scale_2d;
    //         this.set_scale(Scale::from_log2_factor(initial_scale.log2_factor() + delta));
    //         Ok(DragOutcome::Continue)
    //     }))
    // }

    // /// Moves the camera in 2D.
    // fn apply_move(&mut self, movement: Move2D) {
    //     let Move2D { dx, dy } = movement;
    //     let delta: FVec2D = NdVec([r64(dx), r64(dy)]);
    //     self.center += self.scale.units_to_tiles(delta.to_fixedvec());
    // }
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

    let denominator = s1.factor() - s2.factor();
    let numerator = 2.0_f64.ln() * (s1.log2_factor() - s2.log2_factor());
    if numerator.is_zero() || denominator.is_zero() {
        // The expression is undefined at s₁ = s₂, but then the average of s₁
        // and s₂ is trivial.
        s1
    } else {
        Scale::from_factor(-numerator / denominator)
    }
}
