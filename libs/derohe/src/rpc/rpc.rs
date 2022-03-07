#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DataType {
    String,
    Int64,
    Uint64,
    Float64,
    Hash,
    Address,
    Time,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Argument {
    name: String,
    data_type: DataType,
    // Value
}

#[derive(Debug, Clone, PartialEq)]
pub struct Arguments(Vec<Argument>);

impl Arguments {
    pub fn empty() -> Self {
        Arguments(vec![])
    }
}