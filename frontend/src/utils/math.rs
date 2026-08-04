//! 3D math helpers used by the WebGL renderer and glTF utilities.

/// Builds a 4×4 identity matrix.
pub fn mat4_identity() -> [[f32; 4]; 4] {
    [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

/// Multiplies two 4×4 matrices.
pub fn mat4_mul(a: &[[f32; 4]; 4], b: &[[f32; 4]; 4]) -> [[f32; 4]; 4] {
    let mut result = [[0.0_f32; 4]; 4];
    for i in 0..4 {
        for j in 0..4 {
            for k in 0..4 {
                result[i][j] += a[i][k] * b[k][j];
            }
        }
    }
    result
}

/// Transforms a 3D vector by a 4×4 matrix (assuming `w = 1.0`).
#[allow(clippy::many_single_char_names)]
pub fn mat4_mul_vec3(m: &[[f32; 4]; 4], v: [f32; 3]) -> [f32; 3] {
    let x = m[0][0] * v[0] + m[1][0] * v[1] + m[2][0] * v[2] + m[3][0];
    let y = m[0][1] * v[0] + m[1][1] * v[1] + m[2][1] * v[2] + m[3][1];
    let z = m[0][2] * v[0] + m[1][2] * v[1] + m[2][2] * v[2] + m[3][2];
    [x, y, z]
}

/// Converts a column-major 4×4 matrix to a flat `[f32; 16]` row-major array.
pub fn mat4_to_array(m: &[[f32; 4]; 4]) -> [f32; 16] {
    let mut result = [0.0_f32; 16];
    for col in 0..4 {
        for row in 0..4 {
            result[col * 4 + row] = m[row][col];
        }
    }
    result
}

/// Extracts the upper-left 3×3 normal matrix from a 4×4 transform.
pub fn mat4_to_normal_matrix_3x3(m: &[[f32; 4]; 4]) -> [f32; 9] {
    [
        m[0][0], m[1][0], m[2][0], m[0][1], m[1][1], m[2][1], m[0][2], m[1][2], m[2][2],
    ]
}

/// Computes a perspective projection matrix.
pub fn perspective_matrix(fov_y: f32, aspect: f32, near: f32, far: f32) -> [f32; 16] {
    let f = 1.0 / (fov_y * 0.5).tan();
    let nf = 1.0 / (near - far);

    [
        f / aspect,
        0.0,
        0.0,
        0.0,
        0.0,
        f,
        0.0,
        0.0,
        0.0,
        0.0,
        (far + near) * nf,
        -1.0,
        0.0,
        0.0,
        2.0 * far * near * nf,
        0.0,
    ]
}

/// Computes a look-at view matrix.
#[allow(clippy::many_single_char_names)]
pub fn look_at_matrix(eye: [f32; 3], center: [f32; 3], up: [f32; 3]) -> [[f32; 4]; 4] {
    let mut f = [center[0] - eye[0], center[1] - eye[1], center[2] - eye[2]];
    normalize_vec3_mut(&mut f);
    let mut s = cross_vec3(&f, &up);
    normalize_vec3_mut(&mut s);
    let mut u = cross_vec3(&s, &f);
    normalize_vec3_mut(&mut u);

    [
        [s[0], u[0], -f[0], 0.0],
        [s[1], u[1], -f[1], 0.0],
        [s[2], u[2], -f[2], 0.0],
        [
            -dot_vec3(&s, &eye),
            -dot_vec3(&u, &eye),
            dot_vec3(&f, &eye),
            1.0,
        ],
    ]
}

/// Normalizes a 3D vector.
pub fn normalize_vec3(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len > 0.0 {
        [v[0] / len, v[1] / len, v[2] / len]
    } else {
        v
    }
}

fn normalize_vec3_mut(v: &mut [f32; 3]) {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len > 0.0 {
        v[0] /= len;
        v[1] /= len;
        v[2] /= len;
    }
}

/// Computes the cross product of two 3D vectors.
pub fn cross_vec3(a: &[f32; 3], b: &[f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// Computes the dot product of two 3D vectors.
pub fn dot_vec3(a: &[f32; 3], b: &[f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// Converts a `three-d` `Vec3` into a plain `[f32; 3]`.
pub fn vec3_to_array(v: three_d_asset::Vec3) -> [f32; 3] {
    [v.x, v.y, v.z]
}

#[cfg(test)]
mod tests {
    #![allow(clippy::float_cmp)]

    use super::*;

    #[test]
    fn mat4_identity_leaves_vectors_unchanged() {
        let m = mat4_identity();
        assert_eq!(mat4_mul_vec3(&m, [1.0, 2.0, 3.0]), [1.0, 2.0, 3.0]);
    }

    #[test]
    fn mat4_mul_with_identity_is_noop() {
        let m = mat4_identity();
        assert_eq!(mat4_mul(&m, &m), m);
    }

    #[test]
    fn mat4_mul_vec3_applies_translation() {
        let mut m = mat4_identity();
        m[3] = [1.0, 2.0, 3.0, 1.0];
        assert_eq!(mat4_mul_vec3(&m, [0.0, 0.0, 0.0]), [1.0, 2.0, 3.0]);
    }

    #[test]
    fn mat4_to_array_roundtrips_identity() {
        let m = mat4_identity();
        let flat = mat4_to_array(&m);
        // Column-major layout: each contiguous group of four values is a column.
        let expected: [f32; 16] = [
            1.0, 0.0, 0.0, 0.0, // column 0
            0.0, 1.0, 0.0, 0.0, // column 1
            0.0, 0.0, 1.0, 0.0, // column 2
            0.0, 0.0, 0.0, 1.0, // column 3
        ];
        assert_eq!(flat, expected);
    }

    #[test]
    fn perspective_matrix_has_expected_shape() {
        let p = perspective_matrix(std::f32::consts::FRAC_PI_4, 1.0, 0.1, 100.0);
        assert_eq!(p.len(), 16);
        let expected_f = 1.0 / ((std::f32::consts::FRAC_PI_4 / 2.0).tan());
        assert!((p[0] - expected_f).abs() < f32::EPSILON);
        // The perspective matrix stores -1 in the column-major index that maps
        // the view-space Z component to clip-space W.
        assert!((p[11] - (-1.0)).abs() < f32::EPSILON);
    }

    #[test]
    fn look_at_oriented_along_negative_z() {
        let m = look_at_matrix([0.0, 0.0, 1.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        // Forward axis in camera space should point along -Z.
        let forward = [-m[0][2], -m[1][2], -m[2][2]];
        let expected = normalize_vec3([0.0, 0.0, -1.0]);
        assert!((forward[0] - expected[0]).abs() < 1e-6);
        assert!((forward[1] - expected[1]).abs() < 1e-6);
        assert!((forward[2] - expected[2]).abs() < 1e-6);
    }

    #[test]
    fn normalize_vec3_of_unit_vector_is_unchanged() {
        assert_eq!(normalize_vec3([1.0, 0.0, 0.0]), [1.0, 0.0, 0.0]);
    }

    #[test]
    fn normalize_vec3_of_zero_vector_is_zero() {
        assert_eq!(normalize_vec3([0.0, 0.0, 0.0]), [0.0, 0.0, 0.0]);
    }

    #[test]
    fn cross_vec3_of_basis_vectors() {
        assert_eq!(
            cross_vec3(&[1.0, 0.0, 0.0], &[0.0, 1.0, 0.0]),
            [0.0, 0.0, 1.0]
        );
    }

    #[test]
    fn dot_vec3_of_perpendicular_vectors_is_zero() {
        assert!(dot_vec3(&[1.0, 0.0, 0.0], &[0.0, 1.0, 0.0]).abs() < f32::EPSILON);
    }
}
