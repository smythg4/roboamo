use std::collections::{HashMap, HashSet};

use crate::engine::flow_graph::FlowGraph;
use crate::engine::person::{DutyStatus, Person};
use crate::engine::team::{Position, Team};
use std::fmt::{Display, Formatter};

use std::rc::Rc;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct RoleId {
    pub team: String,
    pub qualification: String,
    pub instance: u32,
}

impl From<RoleId> for Position {
    fn from(val: RoleId) -> Self {
        Position {
            qualification: val.qualification,
            instance: val.instance,
        }
    }
}

pub struct AssignmentSolver {
    graph: FlowGraph,

    person_to_node: HashMap<String, usize>,
    node_to_person: HashMap<usize, String>,

    role_to_node: HashMap<RoleId, usize>,
    node_to_role: HashMap<usize, RoleId>,

    team_to_node: HashMap<String, usize>,
    node_to_team: HashMap<usize, String>,

    source_node: usize,
    sink_node: usize,
}

impl AssignmentSolver {
    pub fn new(
        people: &[Person],
        teams: &[Team],
        analysis_date: chrono::NaiveDate,
        assignment_locks: Option<Vec<AssignmentLock>>,
    ) -> Self {
        let num_people = people.len();
        let num_roles = teams
            .iter()
            .map(|t| t.required_positions.len())
            .sum::<usize>();
        let num_teams = teams.len();
        let total_nodes = 1 + num_people + num_roles + num_teams + 1;

        let mut solver = AssignmentSolver {
            graph: FlowGraph::new(total_nodes),
            person_to_node: HashMap::new(),
            node_to_person: HashMap::new(),
            role_to_node: HashMap::new(),
            node_to_role: HashMap::new(),
            team_to_node: HashMap::new(),
            node_to_team: HashMap::new(),
            source_node: 0,
            sink_node: total_nodes - 1,
        };

        solver.build_network(people, teams, analysis_date, assignment_locks);
        solver
    }

    fn build_network(
        &mut self,
        people: &[Person],
        teams: &[Team],
        analysis_date: chrono::NaiveDate,
        assignment_locks: Option<Vec<AssignmentLock>>,
    ) {
        let mut node_idx = 1; // source is 0

        let (locked_people_set, locked_role_ids) = if let Some(locks) = assignment_locks {
            let locked_people: HashSet<String> =
                locks.iter().map(|al| al.person_name.clone()).collect();

            let locked_role_ids: HashSet<RoleId> = locks
                .iter()
                .filter_map(|al| {
                    if let (Some(team), Some(position)) = (&al.team_name, &al.position) {
                        // update this logic to dynamically calculate the instance to lock
                        Some(position.into_role_id(team))
                    } else {
                        None
                    }
                })
                .collect();

            (locked_people, locked_role_ids)
        } else {
            (HashSet::new(), HashSet::new())
        };
        // person nodes
        for person in people {
            if !locked_people_set.contains(&person.name) {
                self.person_to_node.insert(person.name.clone(), node_idx);
                self.node_to_person.insert(node_idx, person.name.clone());

                self.graph.add_edge(self.source_node, node_idx, 1, 0);
                node_idx += 1;
            }
        }

        // role nodes
        for team in teams {
            for position in &team.required_positions {
                let role_id = position.into_role_id(&team.name);

                if !locked_role_ids.contains(&role_id) {
                    self.role_to_node.insert(role_id.clone(), node_idx);
                    self.node_to_role.insert(node_idx, role_id);
                    node_idx += 1;
                }
            }
        }

        // team nodes
        for team in teams {
            self.team_to_node.insert(team.name.clone(), node_idx);
            self.node_to_team.insert(node_idx, team.name.clone());
            node_idx += 1;
        }

        // add edges between layers
        self.add_person_to_role_edges(people, analysis_date);
        self.add_role_to_team_edges(teams);
        self.add_team_to_sink_edges(teams);
    }

    fn add_person_to_role_edges(&mut self, people: &[Person], analysis_date: chrono::NaiveDate) {
        for person in people {
            if let Some(&person_node) = self.person_to_node.get(&person.name) {
                for (role_id, &role_node) in &self.role_to_node {
                    if person.qualifications.contains(&role_id.qualification) {
                        let cost = self.calculate_assignment_cost(person, role_id, analysis_date);

                        self.graph.add_edge(person_node, role_node, 1, cost);
                    }
                }
            }
        }
    }

    fn add_role_to_team_edges(&mut self, teams: &[Team]) {
        for team in teams {
            let team_node = self.team_to_node[&team.name];

            for (role_id, &role_node) in &self.role_to_node {
                if role_id.team == team.name {
                    self.graph.add_edge(role_node, team_node, 1, 0);
                }
            }
        }
    }

    fn add_team_to_sink_edges(&mut self, teams: &[Team]) {
        for team in teams {
            let team_node = self.team_to_node[&team.name];
            let team_capacity = team.required_positions.len() as i32;

            self.graph
                .add_edge(team_node, self.sink_node, team_capacity, 0);
        }
    }

    fn calculate_assignment_cost(
        &self,
        person: &Person,
        role_id: &RoleId,
        analysis_date: chrono::NaiveDate,
    ) -> i32 {
        let mut cost = 0;

        match person.duty_status {
            DutyStatus::TAR => cost += 0,
            DutyStatus::SELRES => cost += 15_000,
        }

        if let Some(prd) = person.prd {
            let days_remaining = (prd - analysis_date).num_days();
            match days_remaining {
                d if d < 0 => cost += 20_000,
                d if d < 90 => cost += 11_000,
                d if d < 180 => cost += 5_000,
                d if d < 365 => cost += 1_000,
                _ => cost += 0,
            }
        }

        if person.raterank.starts_with("AW") {
            cost += 10_000;
        }

        if !person.raterank.starts_with("A") {
            cost += 10_000;
        }

        if person.raterank.ends_with("C")
            || person.raterank.ends_with("CS")
            || person.raterank.ends_with("CM")
        {
            cost += 5_000;
        }

        if person.raterank.ends_with("CM") {
            cost += 5_000;
        }

        if person.raterank.ends_with("CMD") {
            cost += 10_000;
        }

        if ["SFF", "Chief", "F/S QAR"].contains(&role_id.qualification.as_str()) {
            // incentive filling these positions over others
            cost -= 1_000;
        }

        cost
    }

    pub fn solve(&mut self) -> (i32, i32) {
        self.graph
            .min_cost_max_flow(self.source_node, self.sink_node)
    }

    pub fn extract_assignments(&self) -> Vec<FlowAssignment> {
        let mut assignments = vec![];

        for (person_name, &person_node) in &self.person_to_node {
            for &edge_idx in &self.graph.graph[person_node] {
                let edge = &self.graph.edges[edge_idx];
                if edge.flow == 1 {
                    if let Some(role_id) = self.node_to_role.get(&edge.to) {
                        assignments.push(FlowAssignment {
                            person_name: person_name.clone(),
                            team: role_id.team.clone(),
                            position: role_id.clone().into(),
                            manual_override: false,
                        });
                    }
                }
            }
        }
        assignments
    }

    // pub fn into_assignment_plan(self, people: &[Person], teams: &[Team]) -> AssignmentPlan {
    //     let flow_assignments = self.extract_assignments();
    //     let assigned_names: Vec<_> = flow_assignments.iter().map(|a| &a.person_name).collect();

    //     let (_assigned_people, unassigned_people): (Vec<&Person>,Vec<&Person>) = people.iter()
    //         .partition(|p| assigned_names.contains( &&p.get_name().to_string() ));

    //     let assignments = flow_assignments.iter()
    //         .map(|a| {
    //             Assignment {
    //                 person: Rc::new(people.iter().find(|p| p.get_name() == a.person_name).unwrap().clone()),
    //                 team_name: a.team.clone(),
    //                 qualification: a.qualification.clone(),
    //                 score: 1 }
    //         }).collect();

    //     let mut unfilled_positions = vec![];
    //     for team in teams {
    //         for position in &team.required_positions {
    //             let req = position.count;
    //             let have = flow_assignments.iter()
    //                 .filter(|a| a.qualification == position.qualification && a.team == team.name)
    //                 .count();
    //             if have < req {
    //                 unfilled_positions.push((team.name.clone(), position.qualification.clone()))
    //             }
    //         }
    //     }

    //     AssignmentPlan{
    //         unassigned_people: Rc::new(unassigned_people.iter().map(|p| *p).cloned().collect()),
    //         assignments,
    //         unfilled_positions,
    //     }
    // }
}

#[derive(Debug)]
pub struct FlowAssignment {
    pub person_name: String,
    pub team: String,
    pub position: Position,
    pub manual_override: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Assignment {
    pub person: Rc<Person>,
    pub team_name: String,
    pub position: Position,
    pub score: i32,
    pub manual_override: bool,
}

impl Assignment {
    pub fn role_id(&self) -> String {
        self.position.role_id(&self.team_name)
    }
}

impl Display for Assignment {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "<{}> {} as {}",
            &self.score, &self.person, &self.position.qualification
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AssignmentPlan {
    pub assignments: Vec<Assignment>,
    pub unfilled_positions: Vec<(String, String)>,
    pub unassigned_people: Rc<Vec<Person>>,
}

#[derive(Debug, Clone)]
pub struct AssignmentLock {
    pub person_name: String,
    pub team_name: Option<String>,
    pub position: Option<Position>,
}
