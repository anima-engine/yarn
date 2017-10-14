use std::collections::{HashMap, VecDeque};
use std::rc::Rc;

use bincode;

use super::block::Block;
use super::yarn_container::YarnContainer;

pub struct Yarn {
    blocks: VecDeque<Option<Block>>,
    rcs: HashMap<usize, Rc<Block>>,
    indices: HashMap<*const Block, usize>
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

    pub(super) fn tie_rc(&mut self, rc: Rc<Block>) -> usize {
        let ptr = &*rc as *const Block;

        if self.indices.contains_key(&ptr) {
            let index = *self.indices.get(&ptr).unwrap();

            match Rc::try_unwrap(rc) {
                Ok(block) => {
                    self.blocks.insert(index, Some(block));
                    self.indices.remove(&ptr).unwrap();
                }
                Err(_) => ()
            };

            index
        } else {
            let index = self.blocks.len();

            self.blocks.push_back(None);
            self.rcs.insert(index, rc);

            index
        }
    }

    pub(super) fn untie_rc(&mut self, index: usize) -> Option<Rc<Block>> {
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
