//! Optimizations over R1CS
//! 
//! //! This is an untouched copy of the CirC target/r1cs/opt module.
use fxhash::{FxHashMap as HashMap, FxHashSet as HashSet};
use log::debug;

use std::collections::hash_map::Entry;

// use circ::target::r1cs::*;
use super::r1cs::*;
use circ::cfg::CircCfg;
use circ::util::once::OnceQueue;

struct LinReducer {
    r1cs: R1cs,
    uses: HashMap<Var, HashSet<usize>>,
    queue: OnceQueue<usize>,
    /// The maximum size LC (number of non-constant monomials)
    /// that will be used for propagation
    lc_size_thresh: usize,
}

impl LinReducer {
    fn new(mut r1cs: R1cs, lc_size_thresh: usize) -> Self {
        let uses = LinReducer::gen_uses(&r1cs);
        let queue = (0..r1cs.constraints.len()).collect::<OnceQueue<usize>>();
        for c in &mut r1cs.constraints {
            normalize(c);
        }
        Self {
            r1cs,
            uses,
            queue,
            lc_size_thresh,
        }
    }

    // generate a new uses hash
    fn gen_uses(r1cs: &R1cs) -> HashMap<Var, HashSet<usize>> {
        let mut uses: HashMap<Var, HashSet<usize>> =
            HashMap::with_capacity_and_hasher(r1cs.num_vars(), Default::default());
        let mut add = |i: usize, y: &Lc| {
            for x in y.monomials.keys() {
                uses.get_mut(x).map(|m| m.insert(i)).or_else(|| {
                    let mut m: HashSet<usize> = Default::default();
                    m.insert(i);
                    uses.insert(*x, m);
                    None
                });
            }
        };
        for (i, (a, b, c)) in r1cs.constraints.iter().enumerate() {
            add(i, a);
            add(i, b);
            add(i, c);
        }
        uses
    }

    /// Substitute `val` for `var` in constraint with id `con_id`.
    /// Updates uses conservatively (not precisely)
    /// Returns whether a sub happened.
    fn sub_in(&mut self, var: Var, val: &Lc, con_id: usize) -> bool {
        let (a, b, c) = &mut self.r1cs.constraints[con_id];
        let uses = &mut self.uses;
        let mut do_in = |a: &mut Lc| {
            if let Some(sc) = a.monomials.remove(&var) {
                assert_eq!(&a.modulus, &val.modulus);
                a.constant += sc.clone() * &val.constant;
                let tot = a.monomials.len() + val.monomials.len();
                if tot > a.monomials.capacity() {
                    a.monomials.reserve(tot - a.monomials.capacity());
                }
                for (i, v) in &val.monomials {
                    match a.monomials.entry(*i) {
                        Entry::Occupied(mut e) => {
                            let m = e.get_mut();
                            *m += sc.clone() * v;
                            if e.get().is_zero() {
                                uses.get_mut(i).unwrap().remove(&con_id);
                                e.remove_entry();
                            }
                        }
                        Entry::Vacant(e) => {
                            e.insert(sc.clone() * v);
                            uses.get_mut(i).unwrap().insert(con_id);
                        }
                    }
                }
                true
            } else {
                false
            }
        };
        let change_a = do_in(a);
        let change_b = do_in(b);
        let change_c = do_in(c);
        let change = change_a || change_b || change_c;
        self.uses.get_mut(&var).unwrap().remove(&con_id);
        if change {
            normalize(&mut self.r1cs.constraints[con_id]);
        }
        change
    }

    fn clear_constraint(&mut self, i: usize) {
        for v in self.r1cs.constraints[i].0.monomials.keys() {
            self.uses.get_mut(v).unwrap().remove(&i);
        }
        self.r1cs.constraints[i].0.clear();
        for v in self.r1cs.constraints[i].1.monomials.keys() {
            self.uses.get_mut(v).unwrap().remove(&i);
        }
        self.r1cs.constraints[i].1.clear();
        for v in self.r1cs.constraints[i].2.monomials.keys() {
            self.uses.get_mut(v).unwrap().remove(&i);
        }
        self.r1cs.constraints[i].2.clear();
    }

    fn run(mut self) -> R1cs {
        while let Some(con_id) = self.queue.pop() {
            if let Some((var, lc)) = as_linear_sub(&self.r1cs.constraints[con_id], &self.r1cs) {
                if lc.monomials.len() < self.lc_size_thresh {
                    debug!(
                        "Elim: {} -> {}",
                        self.r1cs.idx_to_sig.get_fwd(&var).unwrap(),
                        self.r1cs.format_lc(&lc)
                    );
                    self.clear_constraint(con_id);
                    for use_id in self.uses[&var].clone() {
                        if self.sub_in(var, &lc, use_id)
                            && (self.r1cs.constraints[use_id].0.is_zero()
                                || self.r1cs.constraints[use_id].1.is_zero())
                        {
                            self.queue.push(use_id);
                        }
                    }
                    self.remove_var(var);
                    debug_assert_eq!(0, self.uses[&var].len());
                }
            }
        }
        self.r1cs.constraints.retain(|c| !constantly_true(c));
        self.remove_dead_variables();
        self.r1cs
    }

    fn remove_var(&mut self, var: Var) {
        self.r1cs.idx_to_sig.remove_fwd(&var);
        self.r1cs.terms.remove(&var);
    }

    /// Remove any private dead variables. Run this at the end of optimization.
    fn remove_dead_variables(&mut self) {
        let used: HashSet<Var> = self
            .r1cs
            .constraints
            .iter()
            .flat_map(|c| {
                c.0.monomials
                    .keys()
                    .chain(c.1.monomials.keys().chain(c.2.monomials.keys()))
            })
            .copied()
            .collect();
        let present: HashSet<Var> = self.r1cs.terms.keys().copied().collect();
        for to_remove in present.difference(&used) {
            self.remove_var(*to_remove);
        }
    }
}

fn as_linear_sub((a, b, c): &(Lc, Lc, Lc), r1cs: &R1cs) -> Option<(Var, Lc)> {
    if a.is_zero() || b.is_zero() {
        for i in c.monomials.keys() {
            if r1cs.can_eliminate(*i) {
                let mut lc = c.clone();
                let v = lc.monomials.remove(i).unwrap();
                lc *= v.recip();
                return Some((*i, -lc));
            }
        }
        None
    } else {
        None
    }
}

fn normalize((a, b, c): &mut (Lc, Lc, Lc)) {
    match (a.as_const(), b.as_const()) {
        (Some(ac), _) => {
            *c -= &(b.take() * ac);
            a.clear();
        }
        (_, Some(bc)) => {
            *c -= &(a.take() * bc);
            b.clear();
        }
        _ => {}
    }
}

fn constantly_true((a, b, c): &(Lc, Lc, Lc)) -> bool {
    match (a.as_const(), b.as_const(), c.as_const()) {
        (Some(x), Some(y), Some(z)) => (x.clone() * y - z).is_zero(),
        _ => false,
    }
}

/// Attempt to shrink this system by reducing linearities.
///
/// ## Parameters
///
///   * `lc_size_thresh`: the maximum size LC (number of non-constant monomials) that will be used
///   for propagation. `None` means no size limit.
pub fn reduce_linearities(r1cs: R1cs, cfg: &CircCfg) -> R1cs {
    LinReducer::new(r1cs, cfg.r1cs.lc_elim_thresh).run()
}
