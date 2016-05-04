use std::f32;
// External Library
use cgmath::*;

#[inline]
pub fn rotation_matrix(angle: Rad<f32>, axis: Vector3<f32>) -> Matrix3<f32> {
    let s = if f32::abs(axis.x) < f32::abs(axis.y) && f32::abs(axis.x) < f32::abs(axis.z) {
        Vector3::new(0.0, -axis.z, axis.y)
    } else if f32::abs(axis.y) < f32::abs(axis.x) && f32::abs(axis.y) < f32::abs(axis.z) {
        Vector3::new(-axis.z, 0.0, axis.x)
    } else {
        Vector3::new(-axis.y, axis.z, 0.0)
    };
    let s = s.normalize();
    let t = axis.cross(s);

    let matrix_t = Matrix3::from_cols(axis, s, t);
    let matrix = matrix_t.transpose();
    &matrix_t * &(&Matrix3::from_angle_x(angle) * &matrix)
}

#[inline]
pub fn rotate_matrix(angle: Rad<f32>, axis: Vector3<f32>, matrix: &Matrix4<f32>) -> Matrix4<f32> {


    let mut transform_matrix = Matrix4::from(rotation_matrix(angle, axis));

    transform_matrix.w.x = -matrix.w.x;
    transform_matrix.w.y = -matrix.w.y;
    transform_matrix.w.z = -matrix.w.z;

    let mut translate_back = Matrix4::one();
    translate_back.w.x = matrix.w.x;
    translate_back.w.y = matrix.w.y;
    translate_back.w.z = matrix.w.z;

    &translate_back * &(&transform_matrix * matrix)
}

#[inline]
pub fn to_mat4(mat: &Matrix4<f32>) -> [[f32; 4]; 4] {
    let m = mat;
    [[m[0][0], m[0][1], m[0][2], m[0][3]],
     [m[1][0], m[1][1], m[1][2], m[1][3]],
     [m[2][0], m[2][1], m[2][2], m[2][3]],
     [m[3][0], m[3][1], m[3][2], m[3][3]]]
}

#[inline]
pub fn to_mat3(mat: &Matrix3<f32>) -> [[f32; 3]; 3] {
    let m = mat;
    [[m[0][0], m[0][1], m[0][2]], [m[1][0], m[1][1], m[1][2]], [m[2][0], m[2][1], m[2][2]]]
}

#[inline]
pub fn from_mat4(mat: &Matrix4<f32>) -> [[f32; 3]; 3] {
    [[mat[0][0], mat[0][1], mat[0][2]],
     [mat[1][0], mat[1][1], mat[1][2]],
     [mat[2][0], mat[2][1], mat[2][2]]]
}

#[inline]
pub fn quaternion_to_mat4(q: Quaternion<f32>) -> Matrix4<f32> {
    let mut mat: Matrix4<f32> = Matrix::zero();

    mat.x.x = 1.0 - 2.0 * (q.v.y * q.v.y) - 2.0 * (q.v.z * q.v.z);
    mat.x.y = 2.0 * (q.v.x * q.v.y) + 2.0 * (q.v.z * q.s);
    mat.x.z = 2.0 * (q.v.x * q.v.z) - 2.0 * (q.v.y * q.s);

    mat.y.x = 2.0 * (q.v.x * q.v.y) - 2.0 * (q.v.z * q.s);
    mat.y.y = 1.0 - 2.0 * (q.v.x * q.v.x) - 2.0 * (q.v.z * q.v.z);
    mat.y.z = 2.0 * (q.v.z * q.v.y) + 2.0 * (q.v.x * q.s);

    mat.z.x = 2.0 * (q.v.x * q.v.z) + 2.0 * (q.v.y * q.s);
    mat.z.y = 2.0 * (q.v.z * q.v.y) - 2.0 * (q.v.x * q.s);
    mat.z.z = 1.0 - 2.0 * (q.v.x * q.v.x) - 2.0 * (q.v.y * q.v.y);

    mat.w.w = 1.0;

    mat
}
