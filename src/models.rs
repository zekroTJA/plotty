pub struct Point(pub i64, pub i64);

pub struct Perimeter(pub Point, pub Point);

pub struct Region {
    pub perimeter: Perimeter,
    pub name: String,
    pub owner: u64,
}

impl Perimeter {
    pub fn size(&self) -> i64 {
        (self.0 .1 - self.0 .0) * (self.1 .1 - self.1 .0).abs()
    }
}
