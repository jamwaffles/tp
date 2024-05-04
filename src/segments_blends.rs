//! Trapezoidal trajectory segments blended with arcs.

use crate::{
    arc_blend::ArcBlend,
    trapezoidal_non_zero_3d::{Coord3, Lim, Out, Segment},
};

#[derive(Debug)]
pub enum Item {
    Linear(Segment),
    ArcBlend(ArcBlend),
}

#[derive(Debug)]
pub struct Trajectory {
    pub points: Vec<Coord3>,
    pub blends: Vec<ArcBlend>,
    pub items: Vec<Item>,
    pub limits: Lim,
    pub max_deviation: f32,
    pub total_time: f32,
}

impl Trajectory {
    pub fn new() -> Self {
        Self {
            // points: vec![Coord3::zeros()],
            points: Vec::new(),
            max_deviation: 0.5,
            // blends: vec![ArcBlend::default()],
            blends: Vec::new(),
            items: Vec::new(),
            limits: Lim {
                vel: Coord3::new(5.0, 5.0, 5.0),
                acc: Coord3::new(10.0, 10.0, 10.0),
            },
            total_time: 0.0,
        }
    }

    pub fn push_point(&mut self, new_point: Coord3) {
        match self.points.len() {
            0 => {
                // let b = &mut self.blends[0];
                // *b = ArcBlend::new(
                //     new_point,
                //     b.mid,
                //     b.next,
                //     self.max_deviation,
                //     0.0,
                //     self.limits,
                // );
            }
            // Two points (1 current, 1 new) is one line segment
            1 => {
                // let b = &mut self.blends[0];
                // *b = ArcBlend::new(
                //     b.prev,
                //     b.mid,
                //     new_point,
                //     self.max_deviation,
                //     0.0,
                //     self.limits,
                // );

                let segment = Segment::new(
                    self.points[0],
                    new_point,
                    Coord3::zeros(),
                    Coord3::zeros(),
                    0.0,
                    &self.limits,
                );

                self.items.push(Item::Linear(segment));
            }
            // // 3 points is a properly computed blend and two segments (one new)
            // 2 => {
            //     let Some(Item::Linear(prev_segment)) = self.items.last_mut() else {
            //         panic!("Last item should be a linear segment");
            //     };

            //     let prev = prev_segment.q0();
            //     let mid = prev_segment.q1();

            //     let mut blend =
            //         ArcBlend::new(prev, mid, new_point, self.max_deviation, 0.0, self.limits);

            //     // let prev_segment = Segment::new(
            //     //     prev,
            //     //     blend.arc_start,
            //     //     Coord3::zeros(),
            //     //     Coord3::zeros(),
            //     //     0.0,
            //     //     &self.limits,
            //     // );

            //     blend.start_t = prev_segment.total_time;

            //     let segment = Segment::new(
            //         blend.arc_end,
            //         new_point,
            //         Coord3::zeros(),
            //         Coord3::zeros(),
            //         // Start second segment after blend
            //         blend.start_t + blend.time,
            //         &self.limits,
            //     );

            //     // self.items.push(Item::Linear(prev_segment));
            //     self.items.push(Item::ArcBlend(blend));
            //     self.items.push(Item::Linear(segment));
            // }
            // 3 or more points and we have a blend between the last and the newly added segment
            _ => {
                let Some(Item::Linear(last_segment)) = self.items.last_mut() else {
                    panic!("Last item should be a linear segment");
                };

                let prev = last_segment.q0();
                let mid = last_segment.q1();
                let next = new_point;

                // TODO: Non-zero initial/final velocities
                let mut blend =
                    ArcBlend::new(prev, mid, next, self.max_deviation, 0.0, self.limits);

                // Move last segment's end point to the start of the new blend
                // TODO: Make this a setter instead of clobbering the previous segment completely.
                let prev_segment_replace = Segment::new(
                    last_segment.q0(),
                    blend.arc_start,
                    last_segment.v0(),
                    // Final velocity of previous segment is now the blend start velocity.
                    // Acceleration will be discontinuous.
                    blend.tp(blend.start_t).unwrap().vel,
                    last_segment.start_t,
                    &self.limits,
                );

                // Blend starts at end of shortened previous segment
                blend.start_t = prev_segment_replace.start_t + prev_segment_replace.total_time;

                // Update previous segment's end position to be start of new blend.
                *last_segment = prev_segment_replace;

                // If segments are not colinear, add a new blend between previous linear segment and
                // this new one.
                // TODO: Merge colinear line segments
                if !blend.is_colinear {
                    self.items.push(Item::ArcBlend(blend));
                }

                // Finally push new segment, starting at end of new blend arc
                // TODO: Non-zero velocity
                self.items.push(Item::Linear(Segment::new(
                    blend.arc_end,
                    new_point,
                    // Start velocity of new segment is the same as the end velocity of the blend
                    // arc
                    blend.tp(blend.time).unwrap().vel,
                    Coord3::zeros(),
                    blend.start_t + blend.time,
                    &self.limits,
                )));
            }
        }

        // Very inefficient way of correctly computing total duration
        self.total_time = self
            .items
            .iter()
            .map(|item| match item {
                Item::Linear(line) => line.total_time,
                Item::ArcBlend(blend) => blend.time,
            })
            .sum();

        self.points.push(new_point);
    }

    // Returns true if point belongs to an arc blend
    pub fn tp(&self, t: f32) -> Option<(Out, bool)> {
        if t > self.total_time || t < 0.0 {
            return None;
        }

        // TODO: Filter by start time first. Calling `tp` on every segment until we get a `Some` is
        // hilariously bad.
        self.items.iter().find_map(|item| match item {
            Item::Linear(line) => line.tp(t).map(|out| (out.0, false)),
            Item::ArcBlend(blend) => blend.tp(t).map(|t| (t, true)),
        })
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
