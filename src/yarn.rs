use std::any::Any;
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;

use bincode;

use super::block::{Block, BlockInner};
use super::geometry::GeometryData;
use super::yarn_container::YarnContainer;

macro_rules! match_block {
    ( $slf:ident, $rc:expr, $ptr:expr, $index:expr, $( $id:ident : $typ:ty => $block:expr ),* ) => {
        $(
            if let Ok(rc) = $rc.downcast::<$typ>() {
                match Rc::try_unwrap(rc) {
                    Ok($id) => {
                        $slf.blocks.insert($index, Some($block));
                        $slf.indices.remove($ptr).unwrap();
                    }
                    Err(_) => ()
                };
            }
        )*
    };
}

pub struct Yarn {
    blocks: VecDeque<Option<Block>>,
    rcs: HashMap<usize, Rc<Any>>,
    indices: HashMap<*const (), usize>
}

impl Yarn {
    pub fn new() -> Yarn {
        Yarn {
            blocks: VecDeque::new(),
            rcs: HashMap::new(),
            indices: HashMap::new()
        }
    }

    pub fn len_blocks(&self) -> usize {
        self.blocks.len()
    }

    pub fn is_entangled(&self) -> bool {
        self.blocks.iter().any(|option| option.is_none())
    }

    pub fn into_container(self) -> Result<YarnContainer, Yarn> {
        if self.is_entangled() {
            return Err(self);
        }

        let blocks: Vec<_> = self.blocks.into_iter().map(|option| option.unwrap()).collect();

        if let Ok(bytes) = bincode::serialize(&blocks, bincode::Infinite) {
            Ok(YarnContainer::new(bytes))
        } else {
            Err(
                Yarn {
                    blocks: blocks.into_iter().map(|block| Some(block)).collect(),
                    rcs: HashMap::new(),
                    indices: HashMap::new()
                }
            )
        }
    }

    pub(super) fn tie_block(&mut self, block: Block) -> usize {
        let index = self.blocks.len();

        self.blocks.push_back(Some(block));

        index
    }

    pub(super) fn untie_block(&mut self) -> Option<Block> {
        self.blocks.pop_front().unwrap()
    }

    pub(super) fn tie_rc(&mut self, rc: Rc<Any>) -> usize {
        let ptr = &*rc as *const Any as *const ();

        if self.indices.contains_key(&ptr) {
            let index = *self.indices.get(&ptr).unwrap();

            match_block!(self, rc, &ptr, index,
                geometry: GeometryData => Block(BlockInner::GeometryData(geometry))
            );

            index
        } else {
            let index = self.blocks.len();

            self.blocks.push_back(None);
            self.rcs.insert(index, rc);

            index
        }
    }

    pub(super) fn untie_rc(&mut self, index: usize) -> Option<Rc<Any>> {
        if self.rcs.contains_key(&index) {
            self.rcs.get(&index).map(|rc| rc.clone())
        } else {
            if index < self.blocks.len() {
                let option = self.blocks.get_mut(index).unwrap();
                let rc = Rc::new(option.take().unwrap());

                self.rcs.insert(index, rc.clone());

                Some(rc)
            } else {
                None
            }
        }
    }
}

pub trait Tie: Sized {
    fn tie(self, yarn: &mut Yarn) -> usize;
    fn untie(yarn: &mut Yarn) -> Option<Self>;
}
