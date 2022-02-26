use std::marker::PhantomData;

use super::{system::param::{Res, Local, ResMut}, component::Resource};


pub struct EventId<T> {
    id: usize,
    marker: PhantomData<T>,
}

impl<T> EventId<T> {
    pub fn new(id: usize) -> Self {
        EventId {
            id,
            marker: PhantomData,
        }
    }

    pub fn id(&self) -> usize {
        self.id
    }
}

impl<T> Clone for EventId<T> {
    fn clone(&self) -> Self {
        EventId::new(self.id)
    }
}

pub struct EventInstance<T: Resource> {
    id: EventId<T>,
    event: T,
}


enum State {
    A,
    B,
}

// Resource
pub struct Events<T: Resource> {
    state: State,
    buffer_a: Vec<EventInstance<T>>,
    buffer_b: Vec<EventInstance<T>>,
    event_count: usize,
    a_start_event_count: usize,
    b_start_event_count: usize,
}

impl<T: Resource> Events<T> {
    pub fn new() -> Self {
        Events {
            state: State::A,
            buffer_a: Vec::new(),
            buffer_b: Vec::new(),
            event_count: 0,
            a_start_event_count: 0,
            b_start_event_count: 0,
        }
    }

    pub fn send(&mut self, event: T) {
        let id = EventId::new(self.event_count);
        let event = EventInstance { id, event };
        match self.state {
            State::A => self.buffer_a.push(event),
            State::B => self.buffer_b.push(event),
        };
        self.event_count += 1;
    }

    pub fn send_default(&mut self)
    where
        T: Default
    {
        self.send(Default::default())
    }

    pub fn update(&mut self) {
        match self.state {
            State::A => {
                self.buffer_b.clear();
                self.state = State::B;
                self.b_start_event_count = self.event_count;
            },
            State::B => {                
                self.buffer_a.clear();
                self.state = State::A;
                self.a_start_event_count = self.event_count;
            }
        }
    }

    fn reset_buffer_event_counts(&mut self) {
        self.a_start_event_count = self.event_count;
        self.b_start_event_count = self.event_count;
    }

    pub fn clear(&mut self) {
        self.reset_buffer_event_counts();
        self.buffer_a.clear();
        self.buffer_b.clear();
    }

    pub fn event_reader_len(&self, last_event_count: usize) -> usize {
        let a_len = if last_event_count <= self.a_start_event_count {
            self.buffer_a.len()
        }
        else {
            self.buffer_a.len()
            .checked_sub(last_event_count - self.a_start_event_count)
            .unwrap_or_default()
        };

        let b_len = if last_event_count <= self.b_start_event_count {
            self.buffer_b.len()
        }
        else {
            self.buffer_b.len()
            .checked_sub(last_event_count - self.b_start_event_count)
            .unwrap_or_default()
        };

        a_len + b_len
    }
}

pub struct EventReader<'w, 's, T: Resource> {
    events: Res<'w, Events<T>>,
    last_event_count: Local<'s, (usize, PhantomData<T>)>,
}

impl<'w, 's, T: Resource> EventReader<'w, 's, T> {
    pub fn new(events: Res<'w, Events<T>>, last_event_count: Local<'s, (usize, PhantomData<T>)>) -> Self {
        EventReader {
            events,
            last_event_count,
        }
    }

    pub fn iter<'a>(&'a mut self) -> impl Iterator<Item = (&'a T, EventId<T>)> {
        mod_internal_iterator(&self.events, &mut self.last_event_count.0)
    }
}

fn mod_internal_iterator<'a, T: Resource>(events: &'a Events<T>, last_event_count: &'a mut usize) -> impl Iterator<Item = (&'a T, EventId<T>)> {
    let a_ind = if *last_event_count <= events.a_start_event_count {
        *last_event_count - events.a_start_event_count
    }
    else {
        0
    };

    let b_ind = if *last_event_count <= events.b_start_event_count {
        *last_event_count - events.b_start_event_count
    }
    else {
        0
    };

    let a = events.buffer_a.get(a_ind..).unwrap_or_default();
    let b = events.buffer_a.get(b_ind..).unwrap_or_default();

    let unread_count = a.len() + b.len();
    *last_event_count = events.event_count - unread_count;

    let events_iter = match events.state {
        State::A => b.iter().chain(a.iter()),
        State::B => a.iter().chain(b.iter()),
    };

    events_iter
        .map(|ev| (&ev.event, ev.id.clone()))
        .inspect(|(_, ev_id)| *last_event_count = (ev_id.id + 1).max(*last_event_count))
}

pub struct EventWriter<'w, T: Resource> {
    events: ResMut<'w, Events<T>>
}

impl<'w, T: Resource> EventWriter<'w, T> {
    pub fn new(events: ResMut<'w, Events<T>>) -> Self {
        EventWriter {
            events,
        }
    }

    pub fn send(&mut self, event: T) {
        self.events.send(event)
    }

    pub fn send_default(&mut self)
    where
        T: Default
    {
        self.events.send_default()
    }
}