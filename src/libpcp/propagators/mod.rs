// Copyright 2015 Pierre Talbot (IRCAM)

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

//     http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Propagators are implementations of constraints, a single constraint can be realized by different propagators.
//!
//! We keep the propagator implementations generic over domains implementing specific operations (e.g. intersection or union). Propagators are also implemented to work on variable views, you can always obtain a view from a variable by using the `Identity` view.

pub mod cmp;
pub mod distinct;
pub mod cumulative;

pub use propagators::cmp::*;
pub use propagators::distinct::*;


#[cfg(test)]
pub mod test
{
  use kernel::Trilean;
  use kernel::Trilean::*;
  use propagation::*;
  use gcollections::ops::*;
  use std::fmt::Debug;
  use concept::*;

  fn error_msg<T: Debug, VStore>(test_no: usize, msg: &str,
    prop: &Formula<VStore>, before: &T, after: &T) -> String
  {
    format!("Test {}: {}\n\
             \tPropagator {:?}\n\
             \tBefore: {:?}\n\
             \tAfter: {:?}",
             test_no, msg, prop, before, after)
  }

  fn status_inclusion(s1: Trilean, s2: Trilean) -> bool {
    match s1 {
      True => s2 == True,
      False => s2 == False,
      Unknown => true
    }
  }


  /// contracting: p(d) ⊆ d for any domain d
  fn contracting<VStore>(test_no: usize, vstore: &mut VStore, mut propagator: Formula<VStore>) where
   VStore: VStoreConcept + Clone + Subset
  {
    let d1 = vstore.clone();
    let d1_status = propagator.is_subsumed(vstore);
    let prop_status = propagator.propagate(vstore);
    let d2_status = propagator.is_subsumed(vstore);
    let err1 = error_msg(test_no, "Propagator is not contracting.", &propagator, &d1, &vstore);
    assert!(vstore.is_subset(&d1), err1);
    let err2 = error_msg(test_no, "Propagator status is not monotonic.", &propagator, &d1_status, &d2_status);
    assert!(status_inclusion(d1_status, d2_status), err2);
    if prop_status == false {
      let err3 = error_msg(test_no, "Propagator is not monotonic: the propagation failed but the status is not `False`.",
        &propagator, &d1_status, &d2_status);
      assert!(d2_status == False, err3);
    }
  }

  /// A propagator p is idempotent if and only if for all domains d, p(p(d)) = p(d).
  fn idempotent<VStore>(test_no: usize, vstore: &mut VStore, mut propagator: Formula<VStore>) where
   VStore: VStoreConcept + Clone + Eq
  {
    let prop_status = propagator.propagate(vstore);
    let d1 = vstore.clone();
    let d1_status = propagator.is_subsumed(vstore);
    let prop_status2 = propagator.propagate(vstore);
    let d2_status = propagator.is_subsumed(vstore);
    let err1 = error_msg(test_no, "Propagator is not idempotent.", &propagator, &d1, &vstore);
    assert!(d1 == vstore.clone() && prop_status == prop_status2 && d1_status == d2_status, err1);
  }

  /// It is monotonic if and only if for any two domains d1 and d2, d1 ⊆ d2 implies p(d1) ⊆ p(d2).
  fn monotonic<VStore>(test_no: usize, vstore1: &mut VStore, vstore2: &mut VStore,
    mut propagator: Formula<VStore>) where
   VStore: VStoreConcept + Clone
  {}

  /// sound: for any domain d ∈ Dom and any assignment a ∈ Asn, if {a} ⊆ d, then p({a}) ⊆ p(d)
  fn sound<VStore>(test_no: usize, assignment: &mut VStore, vstore: &mut VStore,
    mut propagator: Formula<VStore>) where
   VStore: VStoreConcept + Clone
  {}


  /// Old test.
  use gcollections::ops::*;
  use kernel::*;
  use variable::VStoreFD;
  use propagation::*;
  use propagation::events::*;
  use interval::interval::*;
  use term::identity::*;
  use variable::store::test::consume_delta;

  type VStore = VStoreFD;
  pub type FDVar = Identity<Interval<i32>>;

  pub fn subsumption_propagate<P>(test_num: u32, mut prop: P, store: &mut VStore,
    before: Trilean, after: Trilean,
    delta_expected: Vec<(usize, FDEvent)>, propagate_success: bool) where
   P: Propagator<VStore> + Subsumption<VStore>
  {
    println!("Test number {}", test_num);
    assert_eq!(prop.is_subsumed(store), before);
    assert_eq!(prop.propagate(store), propagate_success);
    if propagate_success {
      consume_delta(store, delta_expected);
    }
    assert_eq!(prop.is_subsumed(store), after);
  }

  pub fn binary_propagator_test<P, FnProp>(test_num: u32, make_prop: FnProp, x: Interval<i32>, y: Interval<i32>,
    before: Trilean, after: Trilean,
    delta_expected: Vec<(usize, FDEvent)>, propagate_success: bool) where
   P: Propagator<VStore> + Subsumption<VStore>,
   FnProp: FnOnce(FDVar, FDVar) -> P
  {
    let mut store = VStore::empty();
    let x = store.alloc(x);
    let y = store.alloc(y);
    let propagator = make_prop(x, y);
    subsumption_propagate(test_num, propagator, &mut store, before, after, delta_expected, propagate_success);
  }

  pub fn trinary_propagator_test<P, FnProp>(test_num: u32, make_prop: FnProp,
    x: Interval<i32>, y: Interval<i32>, z: Interval<i32>,
    before: Trilean, after: Trilean,
    delta_expected: Vec<(usize, FDEvent)>, propagate_success: bool) where
   P: Propagator<VStore> + Subsumption<VStore>,
   FnProp: FnOnce(FDVar, FDVar, FDVar) -> P
  {
    let mut store = VStore::empty();
    let x = store.alloc(x);
    let y = store.alloc(y);
    let z = store.alloc(z);
    let propagator = make_prop(x, y, z);
    subsumption_propagate(test_num, propagator, &mut store, before, after, delta_expected, propagate_success);
  }

  pub fn nary_propagator_test<P, FnProp>(test_num: u32, make_prop: FnProp, doms: Vec<Interval<i32>>,
    before: Trilean, after: Trilean,
    delta_expected: Vec<(usize, FDEvent)>, propagate_success: bool) where
   P: Propagator<VStore> + Subsumption<VStore>,
   FnProp: FnOnce(Vec<FDVar>) -> P
  {
    let mut store = VStore::empty();
    let vars = doms.into_iter().map(|d| store.alloc(d)).collect();
    let propagator = make_prop(vars);
    subsumption_propagate(test_num, propagator, &mut store, before, after, delta_expected, propagate_success);
  }
}
