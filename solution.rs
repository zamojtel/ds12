use crate::domain::{Action, ClientRef, Edit, EditRequest, Operation, ReliableBroadcastRef};
use module_system::Handler;

impl Operation {
    // Add any methods you need.
    fn transform(&self,other :Operation) -> Operation {

        let result_action = match (&self.action,&other.action) {
            (Action::Insert { idx: i1, ch: c1 }, Action::Insert { idx: i2, ch: c2 }) => {
                if i1<i2 || (i1==i2 && self.process_rank<other.process_rank){
                    Action::Insert { idx: *i1, ch: *c1 }
                }else{
                    Action::Insert { idx: *i1+1, ch: *c1 }
                }
            },
            (Action::Insert { idx: i1, ch: c1 }, Action::Delete { idx: i2}) => {
                if i1 <= i2{
                    Action::Insert { idx: *i1, ch: *c1 }
                }else{
                    Action::Insert { idx: *i1-1, ch: *c1 }
                }
            },
            (Action::Delete { idx: i1 }, Action::Insert { idx: i2, ch: _c2 }) => {
                // Action::Delete { idx: *i1 }
                if i2<=i1 {
                    Action::Delete { idx: *i1+1 }
                }else{
                    Action::Delete { idx: *i1 }
                }
            },
            (Action::Delete { idx: i1 },Action::Delete { idx: i2 })=> {
                if i1 == i2 {
                    Action::Nop
                } else if i2 < i1 {
                    Action::Delete { idx: *i1-1 }
                } else{
                    Action::Delete { idx: *i1 }
                }
            },
            (Action::Nop, _) => Action::Nop,
            (_, Action::Nop) => self.action.clone(),
        };
 
        Operation{
            action:result_action,
            process_rank: self.process_rank
        }
    }
}

/// Process of the system.
pub(crate) struct Process<const N: usize> {
    /// Rank of the process.
    rank: usize,
    /// Reference to the broadcast module.
    broadcast: Box<dyn ReliableBroadcastRef<N>>,
    /// Reference to the process's client.
    client: Box<dyn ClientRef>,
    // Add any fields you need.
}

impl<const N: usize> Process<N> {
    pub(crate) fn new(
        rank: usize,
        broadcast: Box<dyn ReliableBroadcastRef<N>>,
        client: Box<dyn ClientRef>,
    ) -> Self {
        Self {
            rank,
            broadcast,
            client,
            // Add any fields you need.
        }
    }

    // Add any methods you need.
}

#[async_trait::async_trait]
impl<const N: usize> Handler<Operation> for Process<N> {
    async fn handle(&mut self, msg: Operation) {
        todo!("Handle operation issued by other process.");
    }
}

#[async_trait::async_trait]
impl<const N: usize> Handler<EditRequest> for Process<N> {
    async fn handle(&mut self, request: EditRequest) {
        todo!("Handle edit request from the client.");
    }
}
