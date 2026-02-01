//! Random and noise operators: Random, PerlinNoise, PerlinNoise3D, Hash

use flux_core::context::EvalContext;
use flux_core::id::Id;
use flux_core::operator::{InputResolver, Operator};
use flux_core::OperatorMeta;
use flux_macros::Operator;
use crate::register_operators;
use crate::registry::OperatorRegistry;
use flux_core::port::{InputPort, OutputPort};


// ============================================================================
// Hash function (deterministic pseudo-random)
// ============================================================================

/// Fast hash function based on xxHash-like algorithm
fn hash_u32(mut x: u32) -> u32 {
    x = x.wrapping_mul(0x85ebca6b);
    x ^= x >> 13;
    x = x.wrapping_mul(0xc2b2ae35);
    x ^= x >> 16;
    x
}

/// Hash float to range [0, 1]
fn hash_to_float(seed: u32) -> f32 {
    (hash_u32(seed) as f32) / (u32::MAX as f32)
}

/// Combine multiple values into a single seed
fn combine_seeds(a: u32, b: u32) -> u32 {
    hash_u32(a ^ (b.wrapping_mul(0x9e3779b9)))
}

// ============================================================================
// Simple Perlin-like noise implementation
// ============================================================================

fn fade(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + t * (b - a)
}

fn grad2d(hash: u32, x: f32, y: f32) -> f32 {
    let h = hash & 3;
    match h {
        0 => x + y,
        1 => -x + y,
        2 => x - y,
        _ => -x - y,
    }
}

fn grad3d(hash: u32, x: f32, y: f32, z: f32) -> f32 {
    let h = hash & 15;
    let u = if h < 8 { x } else { y };
    let v = if h < 4 { y } else if h == 12 || h == 14 { x } else { z };
    (if h & 1 == 0 { u } else { -u }) + (if h & 2 == 0 { v } else { -v })
}

/// 2D Perlin noise
fn perlin_2d(x: f32, y: f32, seed: u32) -> f32 {
    let xi = x.floor() as i32;
    let yi = y.floor() as i32;

    let xf = x - x.floor();
    let yf = y - y.floor();

    let u = fade(xf);
    let v = fade(yf);

    let aa = hash_u32(combine_seeds(xi as u32, combine_seeds(yi as u32, seed)));
    let ab = hash_u32(combine_seeds(xi as u32, combine_seeds((yi + 1) as u32, seed)));
    let ba = hash_u32(combine_seeds((xi + 1) as u32, combine_seeds(yi as u32, seed)));
    let bb = hash_u32(combine_seeds((xi + 1) as u32, combine_seeds((yi + 1) as u32, seed)));

    let x1 = lerp(grad2d(aa, xf, yf), grad2d(ba, xf - 1.0, yf), u);
    let x2 = lerp(grad2d(ab, xf, yf - 1.0), grad2d(bb, xf - 1.0, yf - 1.0), u);

    (lerp(x1, x2, v) + 1.0) * 0.5 // Normalize to [0, 1]
}

/// 3D Perlin noise
fn perlin_3d(x: f32, y: f32, z: f32, seed: u32) -> f32 {
    let xi = x.floor() as i32;
    let yi = y.floor() as i32;
    let zi = z.floor() as i32;

    let xf = x - x.floor();
    let yf = y - y.floor();
    let zf = z - z.floor();

    let u = fade(xf);
    let v = fade(yf);
    let w = fade(zf);

    let hash_corner = |dx: i32, dy: i32, dz: i32| -> u32 {
        hash_u32(combine_seeds(
            (xi + dx) as u32,
            combine_seeds((yi + dy) as u32, combine_seeds((zi + dz) as u32, seed)),
        ))
    };

    let aaa = hash_corner(0, 0, 0);
    let aba = hash_corner(0, 1, 0);
    let aab = hash_corner(0, 0, 1);
    let abb = hash_corner(0, 1, 1);
    let baa = hash_corner(1, 0, 0);
    let bba = hash_corner(1, 1, 0);
    let bab = hash_corner(1, 0, 1);
    let bbb = hash_corner(1, 1, 1);

    let x1 = lerp(
        grad3d(aaa, xf, yf, zf),
        grad3d(baa, xf - 1.0, yf, zf),
        u,
    );
    let x2 = lerp(
        grad3d(aba, xf, yf - 1.0, zf),
        grad3d(bba, xf - 1.0, yf - 1.0, zf),
        u,
    );
    let y1 = lerp(x1, x2, v);

    let x1 = lerp(
        grad3d(aab, xf, yf, zf - 1.0),
        grad3d(bab, xf - 1.0, yf, zf - 1.0),
        u,
    );
    let x2 = lerp(
        grad3d(abb, xf, yf - 1.0, zf - 1.0),
        grad3d(bbb, xf - 1.0, yf - 1.0, zf - 1.0),
        u,
    );
    let y2 = lerp(x1, x2, v);

    (lerp(y1, y2, w) + 1.0) * 0.5 // Normalize to [0, 1]
}

// ============================================================================
// Random Operator (using derive macro)
// ============================================================================

#[derive(Operator)]
#[operator(name = "Random", category = "Math", description = "Deterministic random value in range")]
#[operator(category_color = [0.35, 0.35, 0.55, 1.0])]
#[allow(dead_code)]
pub struct RandomOp {
    id: Id,
    inputs: [InputPort; 3],
    outputs: [OutputPort; 1],
    #[input(label = "Min", default = 0.0)]
    min: f32,
    #[input(label = "Max", default = 1.0)]
    max: f32,
    #[input(label = "Seed", default = 0)]
    seed: i32,
    #[output(label = "Result")]
    result: f32,
}

impl RandomOp {
    fn compute_impl(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let min = self.get_min(get_input);
        let max = self.get_max(get_input);
        let seed = self.get_seed(get_input) as u32;

        let t = hash_to_float(seed);
        self.set_result(min + t * (max - min));
    }
}

// ============================================================================
// PerlinNoise Operator (using derive macro)
// ============================================================================

#[derive(Operator)]
#[operator(name = "PerlinNoise", category = "Math", description = "2D Perlin noise")]
#[operator(category_color = [0.35, 0.35, 0.55, 1.0])]
#[allow(dead_code)]
pub struct PerlinNoiseOp {
    id: Id,
    inputs: [InputPort; 3],
    outputs: [OutputPort; 1],
    #[input(label = "X", default = 0.0)]
    x: f32,
    #[input(label = "Y", default = 0.0)]
    y: f32,
    #[input(label = "Scale", default = 1.0)]
    scale: f32,
    #[output(label = "Result")]
    result: f32,
}

impl PerlinNoiseOp {
    fn compute_impl(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let x = self.get_x(get_input);
        let y = self.get_y(get_input);
        let scale = self.get_scale(get_input);

        self.set_result(perlin_2d(x * scale, y * scale, 0));
    }
}

// ============================================================================
// PerlinNoise3D Operator (using derive macro)
// ============================================================================

#[derive(Operator)]
#[operator(name = "PerlinNoise3D", category = "Math", description = "3D Perlin noise")]
#[operator(category_color = [0.35, 0.35, 0.55, 1.0])]
#[allow(dead_code)]
pub struct PerlinNoise3DOp {
    id: Id,
    inputs: [InputPort; 4],
    outputs: [OutputPort; 1],
    #[input(label = "X", default = 0.0)]
    x: f32,
    #[input(label = "Y", default = 0.0)]
    y: f32,
    #[input(label = "Z", default = 0.0)]
    z: f32,
    #[input(label = "Scale", default = 1.0)]
    scale: f32,
    #[output(label = "Result")]
    result: f32,
}

impl PerlinNoise3DOp {
    fn compute_impl(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let x = self.get_x(get_input);
        let y = self.get_y(get_input);
        let z = self.get_z(get_input);
        let scale = self.get_scale(get_input);

        self.set_result(perlin_3d(x * scale, y * scale, z * scale, 0));
    }
}

// ============================================================================
// Hash Operator (using derive macro)
// ============================================================================

#[derive(Operator)]
#[operator(name = "Hash", category = "Math", description = "Deterministic hash of value")]
#[operator(category_color = [0.35, 0.35, 0.55, 1.0])]
#[allow(dead_code)]
pub struct HashOp {
    id: Id,
    inputs: [InputPort; 2],
    outputs: [OutputPort; 1],
    #[input(label = "Value", default = 0.0)]
    value: f32,
    #[input(label = "Seed", default = 0)]
    seed: i32,
    #[output(label = "Result")]
    result: f32,
}

impl HashOp {
    fn compute_impl(&mut self, _ctx: &EvalContext, get_input: InputResolver) {
        let value = self.get_value(get_input);
        let seed = self.get_seed(get_input) as u32;

        // Convert float bits to u32 for hashing
        let value_bits = value.to_bits();
        let combined = combine_seeds(value_bits, seed);
        self.set_result(hash_to_float(combined));
    }
}

// ============================================================================
// Registration
// ============================================================================

pub fn register(registry: &OperatorRegistry) {
    register_operators!(
        registry,
        RandomOp,
        PerlinNoiseOp,
        PerlinNoise3DOp,
        HashOp,
    );
}

#[cfg(test)]
mod tests {
    use std::f32::consts::PI;

    use super::*;
    use flux_core::Value;

    fn no_connections(_: Id, _: usize) -> Value {
        Value::Float(0.0)
    }

    #[test]
    fn test_random_deterministic() {
        let mut op = RandomOp::new();
        op.inputs[0].default = Value::Float(0.0);
        op.inputs[1].default = Value::Float(1.0);
        op.inputs[2].default = Value::Int(42);
        let ctx = EvalContext::new();

        op.compute(&ctx, &no_connections);
        let result1 = op.outputs[0].value.as_float().unwrap();

        op.compute(&ctx, &no_connections);
        let result2 = op.outputs[0].value.as_float().unwrap();

        // Same seed should give same result
        assert_eq!(result1, result2);

        // Different seed should give different result
        op.inputs[2].default = Value::Int(43);
        op.compute(&ctx, &no_connections);
        let result3 = op.outputs[0].value.as_float().unwrap();
        assert_ne!(result1, result3);
    }

    #[test]
    fn test_random_range() {
        let mut op = RandomOp::new();
        op.inputs[0].default = Value::Float(10.0);
        op.inputs[1].default = Value::Float(20.0);
        op.inputs[2].default = Value::Int(123);
        let ctx = EvalContext::new();
        op.compute(&ctx, &no_connections);

        let result = op.outputs[0].value.as_float().unwrap();
        assert!((10.0..=20.0).contains(&result));
    }

    #[test]
    fn test_perlin_noise_range() {
        let mut op = PerlinNoiseOp::new();
        op.inputs[2].default = Value::Float(1.0); // Scale
        let ctx = EvalContext::new();

        // Sample at various points
        for i in 0..10 {
            op.inputs[0].default = Value::Float(i as f32 * 0.5);
            op.inputs[1].default = Value::Float(i as f32 * 0.3);
            op.compute(&ctx, &no_connections);

            let result = op.outputs[0].value.as_float().unwrap();
            assert!((0.0..=1.0).contains(&result), "Noise should be in [0, 1], got {}", result);
        }
    }

    #[test]
    fn test_perlin_noise_continuity() {
        let mut op = PerlinNoiseOp::new();
        op.inputs[2].default = Value::Float(1.0);
        let ctx = EvalContext::new();

        // Sample two nearby points
        op.inputs[0].default = Value::Float(1.0);
        op.inputs[1].default = Value::Float(1.0);
        op.compute(&ctx, &no_connections);
        let result1 = op.outputs[0].value.as_float().unwrap();

        op.inputs[0].default = Value::Float(1.01);
        op.inputs[1].default = Value::Float(1.0);
        op.compute(&ctx, &no_connections);
        let result2 = op.outputs[0].value.as_float().unwrap();

        // Nearby samples should be close (continuous)
        assert!((result1 - result2).abs() < 0.1);
    }

    #[test]
    fn test_hash_deterministic() {
        let mut op = HashOp::new();
        op.inputs[0].default = Value::Float(PI);
        op.inputs[1].default = Value::Int(0);
        let ctx = EvalContext::new();

        op.compute(&ctx, &no_connections);
        let result1 = op.outputs[0].value.as_float().unwrap();

        op.compute(&ctx, &no_connections);
        let result2 = op.outputs[0].value.as_float().unwrap();

        assert_eq!(result1, result2);
    }

    #[test]
    fn test_hash_different_values() {
        let mut op = HashOp::new();
        op.inputs[1].default = Value::Int(0);
        let ctx = EvalContext::new();

        op.inputs[0].default = Value::Float(1.0);
        op.compute(&ctx, &no_connections);
        let result1 = op.outputs[0].value.as_float().unwrap();

        op.inputs[0].default = Value::Float(2.0);
        op.compute(&ctx, &no_connections);
        let result2 = op.outputs[0].value.as_float().unwrap();

        assert_ne!(result1, result2);
    }
}
