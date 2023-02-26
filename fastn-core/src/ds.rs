#[derive(serde::Serialize, std::fmt::Debug)]
pub struct LengthList<Item> {
    len: usize,
    items: Vec<IndexyItem<Item>>,
}

#[derive(serde::Serialize, std::fmt::Debug)]
pub struct IndexyItem<Item> {
    index: usize,
    item: Item,
}

impl<Item> LengthList<Item> {
    pub fn from_owned(list: Vec<Item>) -> LengthList<Item> {
        use itertools::Itertools;
        LengthList {
            len: list.len(),
            items: list
                .into_iter()
                .enumerate()
                .map(|(index, item)| IndexyItem { index, item })
                .collect_vec(),
        }
    }
}
