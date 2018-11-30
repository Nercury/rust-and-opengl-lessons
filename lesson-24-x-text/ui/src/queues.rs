use std::collections::BTreeMap;
use std::collections::VecDeque;
use *;

pub struct Queues {
    next_queue_id: Ix,
    queues: BTreeMap<Ix, VecDeque<Effect>>,
}

impl Queues {
    pub fn new() -> Queues {
        Queues {
            next_queue_id: Ix(0),
            queues: BTreeMap::new(),
        }
    }

    pub fn create_queue(&mut self) -> Ix {
        self.queues.insert(self.next_queue_id, VecDeque::new());
        let id = self.next_queue_id;
        self.next_queue_id.inc();
        id
    }

    pub fn delete_queue(&mut self, id: Ix) {
        self.queues.remove(&id);
    }

    pub fn send(&mut self, e: Effect) {
        //trace!("event: {:?}", e);

        for (_, q) in self.queues.iter_mut() {
            q.push_back(e.clone());
        }
    }

    pub fn get_queue_mut(&mut self, id: Ix) -> Option<&mut VecDeque<Effect>> {
        self.queues.get_mut(&id)
    }
}