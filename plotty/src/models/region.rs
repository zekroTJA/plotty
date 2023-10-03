#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Point(pub i64, pub i64);

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Perimeter(pub Point, pub Point);

#[derive(Clone)]
pub struct Region {
    pub perimeter: Perimeter,
    pub name: String,
    pub owner: u64,
}

impl Perimeter {
    pub fn size(&self) -> i64 {
        (self.0 .1 - self.0 .0) * (self.1 .1 - self.1 .0).abs()
    }

    pub fn normalize(&self) -> Perimeter {
        let mut new = self.clone();

        if new.0 .0 > new.1 .0 {
            (new.0, new.1) = (new.1, new.0);
        }

        if new.0 .1 > new.1 .1 {
            (new.0 .1, new.1 .1) = (new.1 .1, new.0 .1)
        }

        new
    }

    pub fn contains_point(&self, p: &Point) -> bool {
        let n = self.normalize();

        n.0 .0 < p.0 && n.1 .0 > p.0 && n.0 .1 < p.1 && n.1 .1 > p.1
    }

    pub fn intersects(&self, other: &Perimeter) -> bool {
        self.intersects_unidirect(other) || other.intersects_unidirect(self)
    }

    fn intersects_unidirect(&self, other: &Perimeter) -> bool {
        let o_corners = [
            &other.0,
            &other.1,
            &Point(other.0 .0, other.1 .1),
            &Point(other.1 .0, other.0 .1),
        ];

        for c in o_corners {
            if self.contains_point(c) {
                return true;
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perimeter_normalize() {
        let p = Perimeter(Point(1, 2), Point(3, 4));

        let exp = Perimeter(Point(1, 2), Point(3, 4));
        assert_eq!(p.normalize(), exp);

        let exp = Perimeter(Point(1, 2), Point(3, 4));
        assert_eq!(p.normalize(), exp);

        let exp = Perimeter(Point(1, 2), Point(3, 4));
        assert_eq!(p.normalize(), exp);

        let exp = Perimeter(Point(1, 2), Point(3, 4));
        assert_eq!(p.normalize(), exp);
    }

    #[test]
    fn perimeter_contains_point() {
        let p = Perimeter(Point(1, 2), Point(4, 5));

        assert!(p.contains_point(&Point(2, 3)));
        assert!(p.contains_point(&Point(3, 3)));
        assert!(p.contains_point(&Point(3, 4)));
        assert!(!p.contains_point(&Point(1, 1)));
        assert!(!p.contains_point(&Point(1, 2)));
        assert!(!p.contains_point(&Point(2, 2)));
        assert!(!p.contains_point(&Point(4, 3)));
    }

    #[test]
    fn perimeter_intersects() {
        let p = Perimeter(Point(1, 2), Point(4, 5));

        let o = Perimeter(Point(3, 4), Point(5, 5));
        assert!(p.intersects(&o));
        assert!(o.intersects(&p));

        let o = Perimeter(Point(2, 3), Point(3, 1));
        assert!(p.intersects(&o));
        assert!(o.intersects(&p));

        let o = Perimeter(Point(2, 3), Point(3, 4));
        assert!(p.intersects(&o));
        assert!(o.intersects(&p));

        let o = Perimeter(Point(2, 0), Point(3, 1));
        assert!(!p.intersects(&o));
        assert!(!o.intersects(&p));

        let o = Perimeter(Point(0, 1), Point(1, 2));
        assert!(!p.intersects(&o));
        assert!(!o.intersects(&p));

        let o = Perimeter(Point(3, 2), Point(2, 1));
        assert!(!p.intersects(&o));
        assert!(!o.intersects(&p));
    }
}
