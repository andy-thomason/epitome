struct Point { x: i32, y: i32 }
enum Shape { Circle(Point, u32), Rect { a: Point, b: Point } }
impl Point {
    fn new(x: i32, y: i32) -> Self { Point { x, y } }
    fn dist2(&self, o: &Point) -> i32 { let dx = self.x - o.x; let dy = self.y - o.y; dx * dx + dy * dy }
}
fn area(s: &Shape) -> u32 {
    match s {
        Shape::Circle(_, r) => 3 * r * r,
        Shape::Rect { a, b } => ((b.x - a.x) * (b.y - a.y)) as u32,
    }
}
fn sum(v: &[u64]) -> u64 {
    let mut t = 0;
    for x in v { t += *x; }
    t
}
fn main() {
    let p = Point::new(1, 2);
    let q = Point::new(4, 6);
    let d = p.dist2(&q);
    let arr = [1, 2, 3];
    let s = sum(&arr);
    let f = 1.5;
    let pair = (d, f);
    let r = loop { break 7u8; };
    let c = |a: i32| a + 1;
    let k = c(d);
}
