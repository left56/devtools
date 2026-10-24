//! Shared quantity chips for pure generators.

/// Chip choices shown in the UI.
pub const CHOICES: &[usize] = &[1, 5, 10, 20, 50];

/// Hard cap for generate-many.
pub const MAX: usize = 100;

pub fn clamp(n: usize) -> usize {
    n.clamp(1, MAX)
}
