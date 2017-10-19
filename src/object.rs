use std::rc::Rc;

use super::block::{Block, BlockInner};
use super::geometry::GeometryData;
use super::yarn::{Tie, Yarn};

#[derive(Debug)]
pub struct Object {
    geometry: Rc<GeometryData>
}

impl Object {
    pub fn new(geometry: Rc<GeometryData>) -> Object {
        Object { geometry }
    }

    pub fn geometry(&self) -> &GeometryData {
        &*self.geometry
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub(super) struct BlockObject {
    geometry_index: usize
}

impl Tie for Object {
    fn into_block(self, yarn: &mut Yarn) -> Block {
        yarn.allocate_block();

        let geometry_index = yarn.tie_rc(self.geometry);

        Block(BlockInner::Object(
            BlockObject { geometry_index }
        ))
    }

    fn from_block(block: Block, yarn: &mut Yarn) -> Option<Self> {
        match block {
            Block(BlockInner::Object(BlockObject { geometry_index })) => {
                let rc = yarn.untie_rc(geometry_index)?;
                Some(
                    Object {
                        geometry: rc.downcast().ok()?
                    }
                )
            }
            _ => unreachable!()
        }
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::*;
    use super::super::geometry::Geometry;
    use super::super::yarn::{Tie, Yarn};

    #[test]
    fn tie_untie() {
        let geometry = Rc::new(
            GeometryData::Geometry(
                Geometry::new(
                    vec![(1.0, 2.0, 3.0); 10],
                    vec![(0.0, 1.0); 6],
                    vec![(1, 2); 16]
                ).unwrap()
            )
        );
        let object1 = Object::new(geometry.clone());
        let object2 = Object::new(geometry);

        let mut yarn = Yarn::new();

        object1.tie(&mut yarn);
        object2.tie(&mut yarn);

        let object1 = Object::untie(&mut yarn).unwrap();
        let object2 = Object::untie(&mut yarn).unwrap();

        match *object1.geometry() {
            GeometryData::Geometry(ref geometry) => {
                assert_eq!(geometry.vertices(), &[(1.0, 2.0, 3.0); 10]);
            }
            _ => unreachable!()
        }
        match *object2.geometry() {
            GeometryData::Geometry(ref geometry) => {
                assert_eq!(geometry.vertices(), &[(1.0, 2.0, 3.0); 10]);
            }
            _ => unreachable!()
        }

        assert_eq!(
            object1.geometry() as *const GeometryData,
            object2.geometry() as *const GeometryData
        );
    }
}
