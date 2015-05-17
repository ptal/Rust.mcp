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

use kernel::*;
use kernel::Trilean::*;
use solver::fd::event::*;
use variable::ops::*;
use interval::ncollections::ops::*;

#[derive(Clone, Copy)]
pub struct XEqY<X, Y>
{
  x: X,
  y: Y
}

impl<X, Y> XEqY<X, Y> {
  pub fn new(x: X, y: Y) -> XEqY<X, Y> {
    XEqY { x: x, y: y }
  }
}

impl<Store, Domain, X, Y> Subsumption<Store> for XEqY<X, Y> where
  X: StoreRead<Store, Value=Domain>,
  Y: StoreRead<Store, Value=Domain>,
  Domain: Bounded + Disjoint
{
  fn is_subsumed(&self, store: &Store) -> Trilean {
    // False:
    // |--|
    //     |--|
    //
    // True:
    // |-|
    // |-|
    //
    // Unknown: Everything else.

    let x = self.x.read(store);
    let y = self.y.read(store);

    if x.is_disjoint(&y) {
      False
    }
    else if x.lower() == y.upper() && x.upper() == y.lower() {
      True
    }
    else {
      Unknown
    }
  }
}

impl<Store, Domain, X, Y> Propagator<Store> for XEqY<X, Y> where
  X: StoreRead<Store, Value=Domain> + StoreMonotonicUpdate<Store, Domain>,
  Y: StoreRead<Store, Value=Domain> + StoreMonotonicUpdate<Store, Domain>,
  Domain: Bounded + Clone,
  Domain: Intersection<Output=Domain>
{
  fn propagate(&mut self, store: &mut Store) -> bool {
    let x = self.x.read(store);
    let y = self.y.read(store);
    let new = x.intersection(&y);
    self.x.update(store, new.clone()) &&
    self.y.update(store, new)
  }
}

impl<X, Y> PropagatorDependencies<FDEvent> for XEqY<X, Y> where
  X: ViewDependencies<FDEvent>,
  Y: ViewDependencies<FDEvent>
{
  fn dependencies(&self) -> Vec<(usize, FDEvent)> {
    let mut deps = self.x.dependencies(FDEvent::Inner);
    deps.append(&mut self.y.dependencies(FDEvent::Inner));
    deps
  }
}

impl<X, Y> DeepClone for XEqY<X, Y> where
  X: Copy,
  Y: Copy
{
  fn deep_clone(&self) -> XEqY<X, Y> {
    *self
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use kernel::*;
  use kernel::Trilean::*;
  use solver::fd::event::*;
  use solver::fd::event::FDEvent::*;
  use interval::interval::*;
  use propagators::test::*;

  #[test]
  fn x_eq_y_test() {
    let dom0_10 = (0,10).to_interval();
    let dom10_20 = (10,20).to_interval();
    let dom5_15 = (5,15).to_interval();
    let dom11_20 = (11,20).to_interval();
    let dom1_1 = (1,1).to_interval();

    x_eq_y_test_one(dom0_10, dom0_10, Unknown, Unknown, vec![], true);
    x_eq_y_test_one(dom0_10, dom10_20, Unknown, True, vec![(0, Assignment), (1, Assignment)], true);
    x_eq_y_test_one(dom5_15, dom10_20, Unknown, Unknown, vec![(0, Bound), (1, Bound)], true);
    x_eq_y_test_one(dom5_15, dom0_10, Unknown, Unknown, vec![(0, Bound), (1, Bound)], true);
    x_eq_y_test_one(dom0_10, dom11_20, False, False, vec![], false);
    x_eq_y_test_one(dom11_20, dom0_10, False, False, vec![], false);
    x_eq_y_test_one(dom1_1, dom0_10, Unknown, True, vec![(1, Assignment)], true);
  }

  fn x_eq_y_test_one(x: Interval<i32>, y: Interval<i32>,
    before: Trilean, after: Trilean,
    delta_expected: Vec<(usize, FDEvent)>, propagate_success: bool)
  {
    binary_propagator_test(XEqY::new, x, y, before, after, delta_expected, propagate_success);
  }
}