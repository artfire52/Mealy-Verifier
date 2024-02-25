use super::event::{Event, Events};

#[derive(Debug)]
pub(crate) struct Premise {
    event: Event,
    counter_event: Events,
}

impl Premise {
    pub(crate) fn new(premise_str: &str) -> Self {
        let split: Vec<&str> = premise_str.split('|').collect();
        let event = Event::new(split[0]);
        let counter_event = Events::from_str(split[1]);
        Premise { event, counter_event }
    }


    pub(crate) fn check_counter_event(&self, event_str: &str) -> bool {
        return self.counter_event.check_all(event_str);
    }


    pub(crate) fn check_event(&self, event_str: &str) -> bool {
        self.event.check(event_str)
    }

}
