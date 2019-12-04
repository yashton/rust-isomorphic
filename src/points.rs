#[derive(Debug)]
pub struct Point(pub i8, pub i8);

#[derive(Debug)]
pub struct BasisVector(pub i8, pub i8);

#[derive(Debug, PartialEq)]
pub struct Interval(pub i8);

pub fn point_to_interval(basis: &BasisVector, point: &Point) -> Interval {
    let BasisVector(bu, bv) = basis;
    let Point(u, v) = point;
    Interval(bu * v + (bv - bu) * u)
}

#[cfg(test)]
mod tests {
    use super::{BasisVector as BV, Point as P, Interval as I, point_to_interval};

    #[test]
    fn interval_at_the_origin_is_zero() {
        assert_eq!(point_to_interval(&BV(3, 4), &P(0, 0)), I(0));
    }

    #[test]
    fn interval_one_step_away() {
        assert_eq!(point_to_interval(&BV(3, 4), &P(0, 1)), I(3));
        assert_eq!(point_to_interval(&BV(3, 4), &P(-1, 0)), I(-1));
        assert_eq!(point_to_interval(&BV(3, 4), &P(1, 0)), I(1));
        assert_eq!(point_to_interval(&BV(3, 4), &P(0, -1)), I(-3));
        assert_eq!(point_to_interval(&BV(3, 4), &P(-1, -1)), I(-4));
        assert_eq!(point_to_interval(&BV(3, 4), &P(1, 1)), I(4));
    }

    #[test]
    fn interval_several_step_away() {
        assert_eq!(point_to_interval(&BV(3, 4), &P(0, 2)), I(6));
        assert_eq!(point_to_interval(&BV(3, 4), &P(0, -2)), I(-6));
        assert_eq!(point_to_interval(&BV(3, 4), &P(2, 0)), I(2));
        assert_eq!(point_to_interval(&BV(3, 4), &P(-2, 0)), I(-2));
        assert_eq!(point_to_interval(&BV(3, 4), &P(1, 2)), I(7));
        assert_eq!(point_to_interval(&BV(3, 4), &P(3, 1)), I(6));
        assert_eq!(point_to_interval(&BV(3, 4), &P(-3, 1)), I(0));
        assert_eq!(point_to_interval(&BV(3, 4), &P(-3, -1)), I(-6));
        assert_eq!(point_to_interval(&BV(3, 4), &P(-1, 2)), I(5));
    }

}
