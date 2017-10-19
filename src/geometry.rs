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
    vertices: Vec<(f32, f32, f32)>,
    uvs: Vec<(f32, f32)>,
    indices: Vec<(usize, usize)>
}

impl Geometry {
    pub fn new(
        vertices: Vec<(f32, f32, f32)>,
        uvs: Vec<(f32, f32)>,
        indices: Vec<(usize, usize)>
    ) -> Option<Geometry> {
        if vertices.iter().any(|&(x, y, z)| !x.is_finite() || !y.is_finite() || !z.is_finite()) {
            return None;
        }

        if uvs.iter().any(|&(u, v)| !u.is_finite() || !v.is_finite()) {
            return None;
        }

        if indices.iter().any(|&(i, j)| i >= vertices.len() || j >= uvs.len()) {
            return None;
        }

        Some(
            Geometry {
                vertices,
                uvs,
                indices
            }
        )
    }

    pub fn vertices(&self) -> &[(f32, f32, f32)] {
        &self.vertices[..]
    }

    pub fn uvs(&self) -> &[(f32, f32)] {
        &self.uvs[..]
    }

    pub fn indices(&self) -> &[(usize, usize)] {
        &self.indices[..]
    }

    pub fn compress(&self) -> GeometryCompressed {
        let encoded: Vec<u8> = bincode::serialize(&self, bincode::Infinite).unwrap();
        let mut data = vec![];

        let mut compressor = BrotliEncoder::new(&encoded[..], 6);
        compressor.read_to_end(&mut data).unwrap();

        GeometryCompressed { data }
    }

    pub fn expand(&self) -> GeometryExpanded {
        let mut vertices = vec![];
        let mut uvs = vec![];

        for &(i, j) in &self.indices {
            vertices.push(self.vertices[i]);
            uvs.push(self.uvs[j]);
        }

        GeometryExpanded {
            vertices,
            uvs
        }
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
    vertices: Vec<(f32, f32, f32)>,
    uvs: Vec<(f32, f32)>
}

impl GeometryExpanded {
    pub fn new(
        vertices: Vec<(f32, f32, f32)>,
        uvs: Vec<(f32, f32)>
    ) -> Option<GeometryExpanded> {
        if vertices.len() != uvs.len() {
            return None;
        }

        if vertices.iter().any(|&(x, y, z)| !x.is_finite() || !y.is_finite() || !z.is_finite()) {
            return None;
        }

        if uvs.iter().any(|&(u, v)| !u.is_finite() || !v.is_finite()) {
            return None;
        }

        Some(
            GeometryExpanded {
                vertices,
                uvs
            }
        )
    }

    pub fn vertices(&self) -> &[(f32, f32, f32)] {
        &self.vertices[..]
    }

    pub fn uvs(&self) -> &[(f32, f32)] {
        &self.uvs[..]
    }

    pub fn condense(&self) -> Geometry {
        let mut vertices: Vec<_> = self.vertices.clone();
        let mut uvs: Vec<_> = self.uvs.clone();

        vertices.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        vertices.dedup_by(|a, b| a.eq(&b));

        uvs.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        uvs.dedup_by(|a, b| a.eq(&b));

        let indices = self.vertices.iter().zip(self.uvs.iter()).map(|(vertex, uv)| {
            let i = vertices.binary_search_by(|probe| probe.partial_cmp(vertex).unwrap()).unwrap();
            let j = uvs.binary_search_by(|probe| probe.partial_cmp(uv).unwrap()).unwrap();

            (i, j)
        }).collect();

        Geometry {
            vertices,
            uvs,
            indices
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compress_decompress() {
        let geometry = Geometry {
            vertices: vec![(1.0, 2.0, 3.0); 10],
            uvs: vec![(0.0, 1.0); 6],
            indices: vec![(1, 2); 16]
        };

        let compressed = geometry.compress();
        let new_geometry = compressed.decompress();

        assert_eq!(new_geometry.vertices, geometry.vertices);
        assert_eq!(new_geometry.uvs, geometry.uvs);
        assert_eq!(new_geometry.indices, geometry.indices);
    }

    #[test]
    fn condense_expand() {
        let expanded = GeometryExpanded {
            vertices: vec![(1.0, 2.0, 3.0); 3],
            uvs: vec![(0.0, 1.0); 3]
        };

        let geometry = expanded.condense();
        let new_expanded = geometry.expand();

        assert_eq!(new_expanded.vertices, expanded.vertices);
        assert_eq!(new_expanded.uvs, expanded.uvs);

        assert_eq!(geometry.vertices.len(), 1);
        assert_eq!(geometry.uvs.len(), 1);
        assert_eq!(geometry.indices, vec![(0, 0), (0, 0), (0, 0)]);
    }
}
