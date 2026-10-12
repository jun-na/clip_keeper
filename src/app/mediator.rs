use super::{controller::Controller, request::Request};
use std::sync::Arc;

/// Marshals requests onto the Slint event loop before dispatching.
pub struct Mediator {
    controller: Arc<Controller>,
}

impl Mediator {
    pub fn new(controller: Arc<Controller>) -> Self {
        Self { controller }
    }

    pub fn send(&self, request: Request) {
        let controller = self.controller.clone();
        if let Err(error) = slint::invoke_from_event_loop(move || controller.dispatch(request)) {
            eprintln!("failed to dispatch request: {error}");
        }
    }
}
