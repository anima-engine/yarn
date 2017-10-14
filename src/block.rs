use super::geometry::GeometryData;
use super::object::BlockObject;

#[derive(Debug, Deserialize, Serialize)]
pub struct Block(pub(super) BlockInner);

#[derive(Debug, Deserialize, Serialize)]
pub(super) enum BlockInner {
    GeometryData(GeometryData),
    Object(BlockObject)
}
