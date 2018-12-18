use prelude::*;

pub type Trig = [Vec2; 3];

pub type Map = Vec<Trig>;

#[derive(Clone)]
pub struct NavMesh {
    points: Vec<Vec2>,
    // could easily be a bitvec
    access: Vec<bool>,
}


pub fn test_coll(map: &Map, p1: Vec2, p2: Vec2, radius: f64) -> Option<Vec2> {
    unimplemented!();
}

fn trig_nav(trig: &Trig, radius: f64) -> [Trig; 6] {
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
            points: Vec::with_capacity(map.len() * 6),
            access: Vec::new(),
        };
        let size = result.points.len();
        for _ in 0..size * size {
            result.access.push(false);
        }
        for i in 0..size {
            result.set_access(i, i, true);
            for j in i+1..size {
                let coll = test_coll(
                    map,
                    result.points[i],
                    result.points[j],
                    radius
                );
                if coll.is_none() {
                    result.set_access(i, j, true);
                    result.set_access(j, i, true);
                }
            }
        }
        result
    }
}
