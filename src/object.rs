use std::rc::Rc;

use super::block::{Block, BlockInner};
use super::geometry::GeometryData;
use super::yarn::{Tie, Yarn};

#[derive(Debug)]
pub struct Object {
    geometry: Rc<GeometryData>
}

#[derive(Debug, Deserialize, Serialize)]
pub(super) struct BlockObject {
    geometry_index: usize
}

impl Tie for Object {
    fn into_block(self, yarn: &mut Yarn) -> Block {
        let geometry_index = yarn.tie_rc(self.geometry);

        Block(BlockInner::Object(
            BlockObject { geometry_index }
        ))
    }

    fn from_block(block: Block, yarn: &mut Yarn) -> Option<Self> {
        match block {
            Block(BlockInner::Object(BlockObject { geometry_index })) => {
                Some(
                    Object {
                        geometry: yarn.untie_rc(geometry_index)?.downcast().unwrap()
                    }
                )
            }
            _ => unreachable!()
        }
    }
}
