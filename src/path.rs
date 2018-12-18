use prelude::*;

pub type Trig = [Vec2; 3];

pub type Map = Vec<Trig>;

pub struct NavMesh {
    points: Vec<Vec2>,
    // could easily be a bitvec
    access: Vec<bool>,
}


pub fn test_coll(map: &Map, p1: Vec2, p2: Vec2, radius: f64) -> Option<Vec2> {
    unimplemented!();
}

fn append_nav(out: &mut Vec<Vec2>, trig: &Trig, radius: f64) {
    unimplemented!();
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
