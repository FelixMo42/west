#[derive(Debug, Clone, Copy)]
pub struct Vec2 <T> {
    pub x: T,
    pub y: T
}

impl<T> Vec2<T> {
    pub fn new(x: T, y: T) -> Vec2<T> {
        return Vec2 { x, y };
    }
}

impl Vec2<i32> {
    pub fn to_u32(&self) -> Vec2<u32> {
        Vec2::new(
            self.x as u32,
            self.y as u32
        )
    }
}
