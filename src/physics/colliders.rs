use bevy::prelude::*;

pub trait Collides<Rhs> {
    fn collides(&self, rhs: &Rhs) -> bool;
}

pub trait CollisionPoint<Rhs>: Collides<Rhs> {
    fn collision_point(&self, rhs: &Rhs) -> Option<Vec3>;
}

macro_rules! reverse {
    ($trait:ident: $fn:ident($lhs:ty, $rhs:ty) -> $ret:ty) => {
        impl $trait<$rhs> for $lhs {
            fn $fn(&self, rhs: &$rhs) -> $ret {
                rhs.$fn(self)
            }
        }
    };
}

#[derive(Debug, Clone, Copy)]
pub struct Plane {
    pub normal: Vec3,
    pub origin: Vec3,
}

impl Collides<Ray> for Plane {
    fn collides(&self, ray: &Ray) -> bool {
        let is_above = ray.origin.dot(self.normal) > 0.0;
        is_above && ray.intersect_plane(self.origin, self.normal).is_some()
    }
}
reverse!(Collides: collides(Ray, Plane) -> bool);

impl CollisionPoint<Ray> for Plane {
    fn collision_point(&self, ray: &Ray) -> Option<Vec3> {
        let is_above = (ray.origin - self.origin).dot(self.normal) > 0.0;
        is_above
            .then(|| ray.intersect_plane(self.origin, self.normal))
            .flatten()
            .map(|distance| ray.get_point(distance))
    }
}
reverse!(CollisionPoint: collision_point(Ray, Plane) -> Option<Vec3>);

#[derive(Debug, Clone, Copy)]
pub struct Rect3d {
    pub origin: Vec3,
    pub extents_a: Vec3,
    pub extents_b: Vec3,
}

impl Rect3d {
    #[allow(dead_code)]
    pub fn gizmo(&self, color: Color, gizmos: &mut Gizmos) {
        let a = self.extents_a * 0.5;
        let b = self.extents_b * 0.5;
        gizmos.linestrip(
            [
                self.origin + a + b,
                self.origin + a - b,
                self.origin - a - b,
                self.origin - a + b,
                self.origin + a + b,
            ],
            color,
        )
    }

    pub fn normal(&self) -> Vec3 {
        self.extents_a.cross(self.extents_b).normalize()
    }
}

impl Collides<Ray> for Rect3d {
    fn collides(&self, ray: &Ray) -> bool {
        self.collision_point(ray).is_some()
    }
}
reverse!(Collides: collides(Ray, Rect3d) -> bool);

impl CollisionPoint<Ray> for Rect3d {
    fn collision_point(&self, ray: &Ray) -> Option<Vec3> {
        let plane = Plane {
            origin: self.origin,
            normal: self.normal(),
        };
        plane.collision_point(ray).filter(|pos| {
            let pos = *pos - self.origin;
            let proj_a = pos.project_onto(self.extents_a);
            let inside_a = proj_a.length_squared() <= (self.extents_a * 0.5).length_squared();
            let proj_b = pos.project_onto(self.extents_b);
            let inside_b = proj_b.length_squared() <= (self.extents_b * 0.5).length_squared();
            inside_a && inside_b
        })
    }
}
reverse!(CollisionPoint: collision_point(Ray, Rect3d) -> Option<Vec3>);
