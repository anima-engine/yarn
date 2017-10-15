use std::any::Any;
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;
use std::str;

use bincode;

use super::block::{Block, BlockInner};
use super::geometry::GeometryData;
use super::object::Object;

macro_rules! match_block {
    ( $slf:ident, $rc:expr, $ptr:expr, $index:expr, [ $typ:ty ] ) => {
        match $rc.downcast::<$typ>() {
            Ok(rc) => {
                match Rc::try_unwrap(rc) {
                    Ok(tie) => {
                        // TODO: fix when NLLs
                        let block = Some(tie.into_block($slf));

                        if $index < $slf.blocks.len() {
                            $slf.blocks[$index] = block;
                        } else {
                            $slf.blocks.push_back(block);
                        }

                        if $slf.indices.contains_key($ptr) {
                            $slf.indices.remove($ptr).unwrap();
                        }
                    }
                    Err(_) => ()
                };
            }
            Err(rc) => {
                $slf.blocks.push_back(None);
                $slf.rcs.insert($index, rc);
            }
        };
    };
    ( $slf:ident, $rc:expr, $ptr:expr, $index:expr, [ $typ:ty, $( $typs:ty ),* ] ) => {
        match $rc.downcast::<$typ>() {
            Ok(rc) => {
                match Rc::try_unwrap(rc) {
                    Ok(tie) => {
                        // TODO: fix when NLLs
                        let block = Some(tie.into_block($slf));

                        if $index < $slf.blocks.len() {
                            $slf.blocks[$index] = block;
                        } else {
                            $slf.blocks.push_back(block);
                        }

                        if $slf.indices.contains_key($ptr) {
                            $slf.indices.remove($ptr).unwrap();
                        }
                    }
                    Err(_) => ()
                };
            }
            Err(rc) => match_block!($slf, rc, $ptr, $index, [ $( $typs ),* ])
        };
    };
}

#[derive(Debug)]
pub struct Yarn {
    blocks: VecDeque<Option<Block>>,
    rcs: HashMap<usize, Rc<Any>>,
    indices: HashMap<*const (), usize>,
    allocated: Vec<usize>
}

impl Yarn {
    pub fn new() -> Yarn {
        Yarn {
            blocks: VecDeque::new(),
            rcs: HashMap::new(),
            indices: HashMap::new(),
            allocated: vec![]
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Yarn> {
        let slice = &bytes[0..4];

        if let Ok(string) = str::from_utf8(slice) {
            if string != "yarn" {
                return None;
            }
        } else {
            return None;
        }

        if let Ok(blocks) = bincode::deserialize::<Vec<Block>>(&bytes[4..]) {
            Some(
                Yarn {
                    blocks: blocks.into_iter().map(|block| Some(block)).collect(),
                    rcs: HashMap::new(),
                    indices: HashMap::new(),
                    allocated: vec![]
                }
            )
        } else {
            None
        }
    }

    pub fn len_blocks(&self) -> usize {
        self.blocks.len()
    }

    pub fn is_entangled(&self) -> bool {
        self.blocks.iter().any(|option| option.is_none())
    }

    pub fn into_bytes(self) -> Result<Vec<u8>, Yarn> {
        if self.is_entangled() {
            return Err(self);
        }

        let mut bytes: Vec<_> = "yarn".as_bytes().into_iter().map(|byte| *byte).collect();

        let blocks: Vec<_> = self.blocks.into_iter().map(|option| option.unwrap()).collect();

        if let Ok(blocks) = bincode::serialize(&blocks, bincode::Infinite) {
            bytes.extend(blocks.into_iter());
            Ok(bytes)
        } else {
            Err(
                Yarn {
                    blocks: blocks.into_iter().map(|block| Some(block)).collect(),
                    rcs: HashMap::new(),
                    indices: HashMap::new(),
                    allocated: vec![]
                }
            )
        }
    }

    pub(super) fn allocate_block(&mut self) {
        // TODO: fix when NLLs
        let index = self.blocks.len();
        self.allocated.push(index);
        self.blocks.push_back(None);
    }

    pub(super) fn tie_block(&mut self, block: Block) -> usize {
        if self.allocated.is_empty() {
            let index = self.blocks.len();

            self.blocks.push_back(Some(block));

            index
        } else {
            let index = self.allocated.pop().unwrap();

            self.blocks[index] = Some(block);

            index
        }
    }

    pub(super) fn untie_block(&mut self) -> Option<Block> {
        let option = self.blocks.iter_mut().find(|option| option.is_some())?;
        Some(option.take().unwrap())
    }

    pub(super) fn tie_rc(&mut self, rc: Rc<Any>) -> usize {
        let ptr = &*rc as *const Any as *const ();

        // TODO: fix when NLLs
        let index = if self.indices.contains_key(&ptr) {
            *self.indices.get(&ptr).unwrap()
        } else {
            self.blocks.len()
        };

        match_block!(self, rc, &ptr, index, [
            GeometryData,
            Object
        ]);

        index
    }

    pub(super) fn untie_rc(&mut self, index: usize) -> Option<Rc<Any>> {
        // TODO: fix when NLLs
        if self.rcs.contains_key(&index) {
            self.rcs.get(&index).map(|rc| rc.clone())
        } else {
            if index < self.blocks.len() {
                // TODO: fix when NLLs
                let block = self.blocks.get_mut(index).unwrap().take().unwrap();

                let rc: Rc<Any> = match block.0 {
                    BlockInner::GeometryData(_) => {
                        Rc::new(GeometryData::from_block(block, self).unwrap())
                    }
                    BlockInner::Object(_) => {
                        Rc::new(Object::from_block(block, self).unwrap())
                    }
                };

                self.rcs.insert(index, rc.clone());

                Some(rc)
            } else {
                None
            }
        }
    }
}

pub trait Tie: Sized {
    fn into_block(self, yarn: &mut Yarn) -> Block;
    fn from_block(block: Block, yarn: &mut Yarn) -> Option<Self>;

    fn tie(self, yarn: &mut Yarn) -> usize {
        // TODO: fix when NLLs
        let block = self.into_block(yarn);
        yarn.tie_block(block)
    }

    fn untie(yarn: &mut Yarn) -> Option<Self> {
        // TODO: fix when NLLs
        let block = yarn.untie_block();
        match block {
            Some(block) => Tie::from_block(block, yarn),
            _ => None
        }
    }
}
