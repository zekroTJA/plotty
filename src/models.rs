pub struct Point(pub i64, pub i64);

pub struct Perimeter(pub Point, pub Point);

pub struct Region {
    pub perimeter: Perimeter,
    pub name: String,
    pub owner: u64,
}
