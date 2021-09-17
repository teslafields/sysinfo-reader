
pub trait SysInfo {
    fn new() -> Self;
    fn read(&mut self);
    fn display(&self);
}

