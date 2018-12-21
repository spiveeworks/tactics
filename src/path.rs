use vecmath;
use piston_window;

use prelude::*;

pub type Trig = [Vec2; 3];

pub type Map = Vec<Trig>;

#[derive(Copy, Clone)]
struct Line {
    point: Vec2,
    dir: Vec2,
}

impl Line {
    fn from_points(p1: Vec2, p2: Vec2) -> Self {
        Line {
            point: p1,
            dir: vec2_sub(p2, p1),
        }
    }

    fn as_fun(self: Self, t: f64) -> Vec2 {
        vec2_add(self.point, vec2_scale(self.dir, t))
    }
}

pub fn unit_is_in_wall(map: &Map, unit: Vec2) -> bool {
    for trig in map {
        if piston_window::math::inside_triangle(*trig, unit) {
            return true;
        }
    }
    false
}

pub fn unit_can_see_pos(map: &Map, unit: Vec2, pos: Vec2) -> bool {
    let test_line = Line::from_points(unit, pos);
    for trig in map {
        for i in 0..3 {
            let j = (i + 1) % 3;
            let map_line = Line::from_points(trig[i], trig[j]);
            if intervals_intersect(test_line, map_line) {
                return false;
            }
        }
    }
    true
}

fn line_intersect_args(line1: Line, line2: Line) -> Option<Vec2> {
    /*
     * line1: {p_1 + t_1v_1 | t_1 \in R}
     * line2: {p_2 + t_2v_2 | t_2 \in R}
     * intersection when p_1 + t_1v_1 = p_2 + t_2v_2
     * p_2 - p_1 = t_1v_1 - t_2v_2
     *   = [t_1 v_1_x - t_2v_2_x; t_1 v_1_y - t_2v_2_y]
     *   = [v_1 <,> -v2] [t_1; t_2]
     * so [t_1; t_2] = (p_2 - p_1) // [v_1 <,> -v2]
     */
    let p = vec2_sub(line2.point, line1.point);
    let mat = [line1.dir, vecmath::vec2_neg(line2.dir), [0.0;2]];
    if vecmath::mat3x2_det(mat) == 0.0 {
        None
    } else {
        let inverse = vecmath::mat3x2_inv(mat);
        Some(vecmath::col_mat3x2_transform_vec2(inverse, p))
    }
}

fn intervals_intersect(line1: Line, line2: Line) -> bool {
    if let Some(vals) = line_intersect_args(line1, line2) {
        0.0 <= vals[0] && vals[0] <= 1.0 &&
        0.0 <= vals[1] && vals[1] <= 1.0
    } else {
        false
    }
}

#[derive(Clone)]
pub struct NavMesh {
    points: Vec<Vec2>,
    // could easily be a bitvec
    access: Vec<bool>,
}

fn intersect_lines(l1: Line, l2: Line) -> Vec2 {
    let val = line_intersect_args(l1, l2).unwrap()[0];
    l1.as_fun(val)
}

pub fn test_coll(map: &Map, p1: Vec2, p2: Vec2, radius: f64) -> Option<Vec2> {
    unimplemented!();
}

fn trig_nav(trig: &Trig, radius: f64) -> [Vec2; 6] {
    let lines = trig_lines(trig, radius);
    hex_points(lines)
}

fn trig_lines(trig: &Trig, radius: f64) -> [Line; 6] {
    unimplemented!();
    /*
     * v1 = p2 - p1, v2 = p3 - p2
     * v1 cross v2 = n * k
     * dir = v1 cross (v1 cross v2) points away from the circle
     * dir /= ||dir||
     * dir *= radius
     *
     * do the same thing but swapping p1 and p3 to get dir2
     * actually dir2 proportional to v2 cross (v1 cross v2) ??
     *
     * then we need 1 more line
     * really we want to turn the trig into a hexagon of 6 lines, then find the
     * corners based on the lines by intersecting adjacent lines
     *
     * so to calculate the bisectors we can just normalize and add the vectors
     * or subtract to get the one moving around the trig
     *
     * but we need the outward direction first to get a point on the line
     *
     * generalize this by saying each of the 6 lines is simply a corner of the
     * trig, plus a vector that drops a perpendicular to the actual line
     *
     * then we need an algorithm to bisect two of those
     */
}

fn hex_points(lines: [Line; 6]) -> [Vec2; 6] {
    let mut out = [[0.0;2]; 6];
    for i in 0..6 {
        let j = (i + 1) % 6;
        out[i] = intersect_lines(lines[i], lines[j]);
    }
    out
}

impl NavMesh {
    fn access_ind(self: &Self, p1: usize, p2: usize) -> usize {
        let size = self.points.len();
        size * p1 + p2
    }
    fn access(self: &Self, p1: usize, p2: usize) -> bool {
        self.access[self.access_ind(p1, p2)]
    }
    fn set_access(self: &mut Self, p1: usize, p2: usize, val: bool) {
        let ind = self.access_ind(p1, p2);
        self.access[ind] = val;
    }

    pub fn generate(map: &Map, radius: f64) -> Self {
        let mut result = NavMesh {
            points: Vec::new(),
            access: Vec::new(),
        };

        result.generate_points(map, radius);
        result.generate_access(map, radius);

        result
    }

    fn generate_points(self: &mut Self, map: &Map, radius: f64) {
        self.points = Vec::with_capacity(map.len() * 6);
        for trig in map {
            for &p in &trig_nav(trig, radius) {
                self.points.push(p);
            }
        }
    }

    fn generate_access(self: &mut Self, map: &Map, radius: f64) {
        self.access.clear();
        let size = self.points.len();
        self.access.reserve(size * size);
        for _ in 0..size * size {
            self.access.push(false);
        }

        for i in 0..size {
            self.set_access(i, i, true);
            for j in i+1..size {
                let coll = test_coll(
                    map,
                    self.points[i],
                    self.points[j],
                    radius
                );
                if coll.is_none() {
                    self.set_access(i, j, true);
                    self.set_access(j, i, true);
                }
            }
        }
    }
}
