// Equivalent to pch.h

pub fn clamp<T: std::cmp::Ord + Copy>(n: &T, lower: &T, upper: &T) -> T {
    std::cmp::max(*lower, std::cmp::min(*n, *upper))
}
