use std::io::Read;

use bincode;
use brotli2::read::{BrotliEncoder, BrotliDecoder};

use super::block::{Block, BlockInner};
use super::yarn::{Tie, Yarn};

#[derive(Debug, Deserialize, Serialize)]
pub enum GeometryData {
    Geometry(Geometry),
    GeometryCompressed(GeometryCompressed),
    GeometryExpanded(GeometryExpanded)
}

impl Tie for GeometryData {
    fn into_block(self, _: &mut Yarn) -> Block {
        Block(BlockInner::GeometryData(self))
    }

    fn from_block(block: Block, _: &mut Yarn) -> Option<Self> {
        match block {
            Block(BlockInner::GeometryData(data)) => Some(data),
            _ => unreachable!()
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Geometry {
    vertices: Vec<f32>,
    uvs: Vec<f32>,
    indices: Vec<usize>
}

impl Geometry {
    pub fn compress(&self) -> GeometryCompressed {
        let encoded: Vec<u8> = bincode::serialize(&self, bincode::Infinite).unwrap();
        let mut data = vec![];

        let mut compressor = BrotliEncoder::new(&encoded[..], 6);
        compressor.read_to_end(&mut data).unwrap();

        GeometryCompressed { data }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GeometryCompressed {
    data: Vec<u8>
}

impl GeometryCompressed {
    pub fn decompress(&self) -> Geometry {
        let mut encoded = vec![];

        let mut decompressor = BrotliDecoder::new(&self.data[..]);
        decompressor.read_to_end(&mut encoded).unwrap();

        bincode::deserialize(&encoded[..]).unwrap()
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GeometryExpanded {
    pub vertices: Vec<f32>,
    pub uvs: Vec<f32>
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compress_decompress() {
        let geometry = Geometry {
            vertices: vec![1.0; 10],
            uvs: vec![0.0; 6],
            indices: vec![2; 16]
        };

        let compressed = geometry.compress();
        let new_geometry = compressed.decompress();

        assert_eq!(new_geometry.vertices, geometry.vertices);
        assert_eq!(new_geometry.uvs, geometry.uvs);
        assert_eq!(new_geometry.indices, geometry.indices);
    }
}
