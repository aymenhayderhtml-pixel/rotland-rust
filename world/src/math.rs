/// Linear interpolation between `from` and `to` by `weight`.
/// Matches Godot's `Mathf.Lerp`.
#[inline]
pub fn lerp(from: f32, to: f32, weight: f32) -> f32 {
    from + (to - from) * weight
}

/// Hermite interpolation between `from` and `to`.
/// Matches Godot's `Mathf.SmoothStep` behavior, including when `from > to`.
#[inline]
pub fn smooth_step(from: f32, to: f32, s: f32) -> f32 {
    if (from - to).abs() <= f32::EPSILON {
        return if from <= to {
            if s <= from {
                0.0
            } else {
                1.0
            }
        } else if s <= to {
            1.0
        } else {
            0.0
        };
    }
    let t = ((s - from) / (to - from)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
