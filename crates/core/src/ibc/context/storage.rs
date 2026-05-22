use crate::error::Error;

pub trait SorobanStorage: Send + Sync {
    fn read_bytes(&self, key: &[u8]) -> Result<Option<Vec<u8>>, Error>;
    fn write_bytes(&mut self, key: &[u8], value: Vec<u8>) -> Result<(), Error>;
    fn delete(&mut self, key: &[u8]) -> Result<(), Error>;
}
