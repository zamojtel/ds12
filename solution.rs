use crate::domain::{Action, ClientRef, Edit, EditRequest, Operation, ReliableBroadcastRef};
use module_system::Handler;

impl Operation {
    // Add any methods you need.
    fn transform(&self,other :Operation) -> Operation {
        let result_action = match (&self.action,&other.action) {
            (Action::Insert { idx: i1, ch: c1 }, Action::Insert { idx: i2, ch: _c2 }) => {
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
    current_round: u64,
    log: Vec<Operation>,
    queue: Vec<Operation>,
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
            current_round: 0,
            log: Vec::new(),
            queue: Vec::new(),
        }
    }

    // Add any methods you need.
}

#[async_trait::async_trait]
impl<const N: usize> Handler<Operation> for Process<N> {
    async fn handle(&mut self, msg: Operation) {
        let mut msg_round = 0;
        let my_round = self.log.len() / N;
        for l in &self.log {
            if msg.process_rank == l.process_rank{
                msg_round+=1;
            }
        }

        if msg_round > my_round{
            self.queue.push(msg);
        }else if msg_round == my_round {
            let mut temp_msg = msg;
            let start = my_round * N;
            
            for op in &self.log[start..] {
                temp_msg = temp_msg.transform(op.clone());
            }

            self.log.push(temp_msg);
            let mut i =0;
            while i < self.queue.len() {
                let mut current_msg_round = 0;
                let current_msg = self.queue[i].clone();

                for l in &self.log {
                    if current_msg.process_rank == l.process_rank{
                        current_msg_round+=1;   
                    }
                }
                if current_msg_round == self.log.len() /N {
                    let mut valid_msg = self.queue.remove(i);
                    let start = current_msg_round * N;
            
                    for op in &self.log[start..] {
                        valid_msg = valid_msg.transform(op.clone());
                    }

                    self.log.push(valid_msg);
                }else{
                    i+=1;
                }
            }
        }else{
            // we ignore the message
        }
    }
}

#[async_trait::async_trait]
impl<const N: usize> Handler<EditRequest> for Process<N> {
    async fn handle(&mut self, request: EditRequest) {

        let mut temp_operation = Operation{
            action: request.action,
            process_rank: N+1,
        };

        let start = (request.num_applied).min(self.log.len());

        for op in &self.log[start..] {
            temp_operation = temp_operation.transform(op.clone());
        }
        
        temp_operation.process_rank = self.rank;  
        self.log.push(temp_operation.clone());
        self.broadcast.send(temp_operation.clone()).await;
        self.client.send(Edit { 
            action: temp_operation.action,
        }).await;
    }
}
