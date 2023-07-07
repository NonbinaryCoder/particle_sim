use bevy::prelude::*;

pub trait Collides<Rhs> {
    fn collides(&self, rhs: &Rhs) -> bool;
}

pub trait CollisionPoint<Rhs>: Collides<Rhs> {
    fn collision_point(&self, rhs: &Rhs) -> Option<Vec3>;
}

pub struct Plane {
    pub normal: Vec3,
    pub origin: Vec3,
}

impl Collides<Ray> for Plane {
    fn collides(&self, ray: &Ray) -> bool {
        let is_above = ray.origin.dot(self.normal) > 0.0;
        // let is_looking_at_plane = self.normal.dot(ray.direction) < 0.0;
        // is_above && is_looking_at_plane
        is_above && ray.intersect_plane(self.origin, self.normal).is_some()
    }
}

impl CollisionPoint<Ray> for Plane {
    fn collision_point(&self, ray: &Ray) -> Option<Vec3> {
        // self.collides(ray).then(|| {
        //     let perp = self.normal.any_orthonormal_vector();
        //     let plane_pos = self.normal * self.height;
        //     let distance =
        //         ((plane_pos - ray.origin).dot(self.normal)) / (ray.direction.dot(self.normal));
        //     ray.intersect_plane(, )
        // })
        let is_above = ray.origin.dot(self.normal) > 0.0;
        is_above
            .then(|| ray.intersect_plane(self.origin, self.normal))
            .flatten()
            .map(|distance| ray.get_point(distance))
    }
}

pub struct Rect3d {
    pub origin: Vec3,
    pub extents_a: Vec3,
    pub extents_b: Vec3,
}

impl Collides<Ray> for Rect3d {
    fn collides(&self, ray: &Ray) -> bool {
        self.collision_point(ray).is_some()
    }
}

impl CollisionPoint<Ray> for Rect3d {
    fn collision_point(&self, ray: &Ray) -> Option<Vec3> {
        let plane = Plane {
            origin: self.origin,
            normal: self.extents_a.cross(self.extents_b).normalize(),
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
