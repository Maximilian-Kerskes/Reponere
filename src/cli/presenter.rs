use crate::handlers::events::event::Event;

pub struct Presenter;

impl Presenter {
    pub fn display<E: Event + ?Sized>(event: &E) {
        println!("{}", event.message());
    }
}
