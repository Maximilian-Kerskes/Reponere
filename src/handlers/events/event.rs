pub trait Event {
    fn message(&self) -> String;
}
