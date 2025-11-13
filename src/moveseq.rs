use std::{collections::VecDeque, fmt};

use cubing::alg::{Alg, AlgNode, Move};

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct MoveSeq(VecDeque<Move>);

impl fmt::Display for MoveSeq {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut is_first = true;
        for m in &self.0 {
            if is_first {
                is_first = false;
            } else {
                write!(f, " ")?;
            }
            write!(f, "{m}")?;
        }
        Ok(())
    }
}

impl PartialOrd for MoveSeq {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for MoveSeq {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        Iterator::cmp(self.cmp_key(), other.cmp_key())
    }
}

impl MoveSeq {
    fn cmp_key(&self) -> impl Iterator<Item = impl Ord> {
        self.0.iter().map(|m| {
            (
                match &m.quantum.prefix {
                    Some(cubing::alg::MovePrefix::Layer(move_layer)) => {
                        [None, Some(move_layer.layer)]
                    }
                    Some(cubing::alg::MovePrefix::Range(move_range)) => {
                        [Some(move_range.inner_layer), Some(move_range.outer_layer)]
                    }
                    None => [None; 2],
                },
                &m.quantum.family,
                m.amount,
            )
        })
    }

    pub fn new() -> Self {
        Self::default()
    }
    pub fn from_alg(alg: &Alg) -> Self {
        let mut ret = Self::new();
        ret.extend_from_alg(alg);
        ret
    }
    pub fn push_back(&mut self, m: Move) {
        if let Some(last) = self.0.iter_mut().last()
            && last.quantum.family == m.quantum.family
        {
            last.amount += m.amount;
            if last.amount == 0 {
                self.0.pop_back();
            }
            return;
        }
        self.0.push_back(m);
    }
    pub fn pop_front_if_fam(&mut self, family: &str) {
        if self.first().is_some_and(|m| m.quantum.family == family) {
            self.0.pop_front();
        }
    }
    pub fn pop_front_if_matches(&mut self, moves: &MoveSeq) -> bool {
        let mut self_iter = self.0.iter();
        for m in moves.iter() {
            if self_iter.next().is_none_or(|s| s != m) {
                return false;
            }
        }

        for _ in 0..moves.len() {
            self.0.pop_front();
        }
        true
    }
    pub fn first(&self) -> Option<&Move> {
        self.0.iter().next()
    }
    fn extend_from_alg(&mut self, alg: &Alg) {
        for node in &alg.nodes {
            match node {
                AlgNode::MoveNode(m) => self.push_back(m.clone()),
                AlgNode::GroupingNode(grouping) if grouping.amount.is_positive() => {
                    for _ in 0..grouping.amount {
                        self.extend_from_alg(&grouping.alg);
                    }
                }
                AlgNode::GroupingNode(grouping) if grouping.amount.is_negative() => {
                    let a = grouping.alg.invert();
                    for _ in 0..grouping.amount.abs() {
                        self.extend_from_alg(&a);
                    }
                }
                AlgNode::CommutatorNode(commutator) => {
                    self.extend_from_alg(&commutator.a);
                    self.extend_from_alg(&commutator.b);
                    self.extend_from_alg(&commutator.a.invert());
                    self.extend_from_alg(&commutator.b.invert());
                }
                AlgNode::ConjugateNode(conjugate) => {
                    self.extend_from_alg(&conjugate.a);
                    self.extend_from_alg(&conjugate.b);
                    self.extend_from_alg(&conjugate.a.invert());
                }
                _ => (),
            }
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Move> {
        self.0.iter()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
