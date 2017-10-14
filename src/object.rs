use std::rc::Rc;

use super::block::{Block, BlockInner};
use super::yarn::{Tie, Yarn};

#[derive(Debug)]
pub struct Object {
    geometry: Rc<Block>
}

#[derive(Debug, Deserialize, Serialize)]
pub(super) struct BlockObject {
    geometry_index: usize
}

impl Tie for Object {
    fn tie(self, yarn: &mut Yarn) -> usize {
        let geometry_index = yarn.tie_rc(self.geometry);

        yarn.tie_block(
            Block(BlockInner::Object(
                BlockObject { geometry_index }
            ))
        )
    }

    fn untie(yarn: &mut Yarn) -> Option<Object> {
        match yarn.untie_block() {
            Some(Block(BlockInner::Object(BlockObject { geometry_index }))) => {
                Some(Object { geometry: yarn.untie_rc(geometry_index)? })
            }
            _ => None
        }
    }
}
