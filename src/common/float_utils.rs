use float_eq::{assert_float_eq, assert_float_ne, float_eq, float_ne};
use crate::common::constants;

pub(crate) fn float_eq(a: f32, b: f32) -> bool {
    float_eq!(a, b, rmax <= constants::FLOAT_EPSILON)
}

#[allow(dead_code)]
pub(crate) fn float_neq(a: f32, b: f32) -> bool {
    float_ne!(a, b, rmax <= constants::FLOAT_EPSILON)
}

pub(crate) fn float_leq(a: f32, b: f32) -> bool {
    if a < b || float_eq!(a, b, rmax <= constants::FLOAT_EPSILON) {
        return true;
    }
    false
}

pub(crate) fn float_geq(a: f32, b: f32) -> bool {
    if a > b || float_eq!(a, b, rmax <= constants::FLOAT_EPSILON) {
        return true;
    }
    false
}

pub fn assert_float_eq(a: f32, b: f32) {
    assert_float_eq!(a, b, rmax <= constants::FLOAT_EPSILON);
}

pub fn assert_float_ne(a: f32, b: f32) {
    assert_float_ne!(a, b, rmax <= constants::FLOAT_EPSILON);
}

#[allow(dead_code)]
pub(crate) fn assert_float_leq(a: f32, b: f32) {
    assert!(float_leq(a, b));
}

#[allow(dead_code)]
pub(crate) fn assert_float_geq(a: f32, b: f32) {
    assert!(float_geq(a, b));
}