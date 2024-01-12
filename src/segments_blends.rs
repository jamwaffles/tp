//! Trapezoidal trajectory segments blended with arcs.

use kiss3d::loader::obj::Coord;

use crate::{
    arc_blend::ArcBlend,
    trapezoidal_non_zero_3d::{Coord3, Lim, Segment},
};

#[derive(Debug)]
pub enum Item {
    Point(Coord3),
    // Linear(Segment),
    ArcBlend(ArcBlend),
}

#[derive(Debug)]
pub struct Trajectory {
    npoints: usize,
    points: Vec<Coord3>,
    blends: Vec<ArcBlend>,
    limits: Lim,
    max_deviation: f32,
}

impl Trajectory {
    pub fn new() -> Self {
        Self {
            points: vec![Coord3::zeros()],
            npoints: 0,
            max_deviation: 0.5,
            blends: vec![ArcBlend::default()],
            limits: Lim {
                vel: Coord3::new(5.0, 5.0, 5.0),
                acc: Coord3::new(10.0, 10.0, 10.0),
            },
        }
    }

    pub fn push_point(&mut self, new_point: Coord3) {
        match self.npoints {
            0 => {
                let b = &mut self.blends[0];
                *b = ArcBlend::new(new_point, b.mid, b.next, self.max_deviation);
            }
            1 => {
                let b = &mut self.blends[0];
                *b = ArcBlend::new(b.prev, new_point, b.next, self.max_deviation);
            }
            _ => {
                let b = self.blends.last_mut().expect("No blends?");
                *b = ArcBlend::new(b.prev, b.mid, new_point, self.max_deviation);
            }
        }

        self.npoints += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn three_points() {
        let p1 = Coord3::new(0.0, 0.0, 0.0);
        let p2 = Coord3::new(3.0, 2.0, 0.0);
        let p3 = Coord3::new(5.0, 1.0, 0.0);

        let mut traj = Trajectory::new();

        traj.push_point(p1);
        traj.push_point(p2);
        traj.push_point(p3);

        dbg!(traj);
    }
}
