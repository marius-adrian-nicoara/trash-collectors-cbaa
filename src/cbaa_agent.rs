use crate::params;
use quicksilver::geom::{Vector, Rectangle};
use argminmax::ArgMinMax;

#[derive(Clone, Debug)]
pub struct Agent {
    pub id: usize,
    pub position: Vector,
    pub assignment: usize,
    pub costs: Vec<f32>,
    pub task_status: [u32; params::NR_TASKS],
    pub bid_list: [f32; params::NR_TASKS]
}

impl Agent {
    pub fn compute_costs(&mut self, task_list: &Vec<Rectangle>) {
        for i in 0..params::NR_TASKS {
            self.costs.push(self.position.distance(task_list[i].pos)*(-1.0));
        }
    }

    pub fn get_task(&mut self) {
        
        // bid only if no current assignment
        if self.assignment == params::NO_TASK {

            //get ID of min cost
            let task_id = self.costs.argmax();

            // place bid
            self.bid_list[task_id] = self.costs[task_id];

            // assign task to self and hope for the best
            self.assignment = task_id;
            self.task_status[task_id] = 1;
        }
    }

    pub fn update_task(&mut self, neighbors: &mut Vec<Agent>) -> bool {
        let prev_task_status = self.task_status;

        // get bids for each task
        for task_id in 0..params::NR_TASKS {

            let mut task_bids: Vec<f32> = vec![self.bid_list[task_id]];

            // get bids from all robots
            for neighbor_id in 0..neighbors.len() {
                let bid = neighbors[neighbor_id].bid_list[task_id];
                task_bids.push(bid);
            }

            // determine winner
            let winner = task_bids.argmax();

            // bid of current agent(self) is placed at index 0
            if winner != 0 {
                // remove assignment, better luck next time
                if self.assignment == task_id {
                    
                    self.assignment = params::NO_TASK;
                }

                // ignore task in the next round
                self.costs[task_id] = f32::NEG_INFINITY;

                // update task status and bid list
                self.task_status[task_id] = 0;
                self.bid_list[task_id] = task_bids[winner];
            }
        }

        // if task status has not changed, the agent converged
        self.task_status == prev_task_status
    }
}