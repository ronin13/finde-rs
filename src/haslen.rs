/// Trait defines the interface so that len() method
/// required by scheduler is supported
pub trait HasLen: std::marker::Send + 'static {
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
