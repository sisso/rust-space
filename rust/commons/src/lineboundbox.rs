/// Take a list of segments and do "collision" check. Kind of AABB in a line
pub struct LineBoundBox {
    segments: Vec<(f32, f32)>,
}

impl Default for LineBoundBox {
    fn default() -> Self {
        LineBoundBox {
            segments: Default::default(),
        }
    }
}

impl LineBoundBox {
    pub fn add(&mut self, min: f32, max: f32) -> bool {
        if !self.is_valid(min, max) {
            return false;
        }

        self.segments.push((min, max));
        true
    }

    pub fn is_valid(&self, min: f32, max: f32) -> bool {
        for (smin, smax) in &self.segments {
            if min >= *smin && min <= *smax {
                return false;
            }

            if max >= *smin && max <= *smax {
                return false;
            }

            if min < *smin && max > *smax {
                return false;
            }
        }

        true
    }
}

#[test]
fn test_lineboundbox() {
    let mut lbb = LineBoundBox::default();

    assert!(lbb.is_valid(1.0, 3.0));
    assert!(lbb.add(1.0, 3.0));
    assert!(!lbb.is_valid(0.0, 1.0));
    assert!(lbb.is_valid(0.0, 0.9));
    assert!(lbb.add(0.0, 0.9));
    assert!(lbb.is_valid(0.91, 0.95));
    assert!(lbb.add(5.0, 9.9));
    assert!(!lbb.add(8.0, 9.0));
}

#[test]
fn test_lineboundbox_new_segment_around_all_others() {
    let mut lbb = LineBoundBox::default();

    assert!(lbb.add(1.0, 3.0));
    assert!(!lbb.is_valid(0.0, 10.0));
}
