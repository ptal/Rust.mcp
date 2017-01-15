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
use model::*;
use propagators::PropagatorKind;
use propagators::XEqY;
use propagation::*;
use propagation::events::*;
use gcollections::ops::*;
use gcollections::*;
use concept::*;
use std::ops::Not;

#[derive(Debug)]
pub struct XNeqY<VStore>
{
  x: Var<VStore>,
  y: Var<VStore>
}

impl<VStore> PropagatorKind for XNeqY<VStore> {}

impl<VStore> XNeqY<VStore> {
  pub fn new(x: Var<VStore>, y: Var<VStore>) -> Self {
    XNeqY { x: x, y: y }
  }
}

impl<VStore> Clone for XNeqY<VStore> where
 VStore: Collection
{
  fn clone(&self) -> Self {
    XNeqY::new(self.x.bclone(), self.y.bclone())
  }
}

impl<VStore> DisplayStateful<Model> for XNeqY<VStore>
{
  fn display(&self, model: &Model) {
    self.x.display(model);
    print!(" != ");
    self.y.display(model);
  }
}

impl<VStore> Not for XNeqY<VStore> {
  type Output = XEqY<VStore>;
  fn not(self) -> Self::Output {
    XEqY::new(self.x, self.y)
  }
}

impl<VStore> Subsumption<VStore> for XNeqY<VStore> where
 VStore: Collection,
 XEqY<VStore>: Subsumption<VStore>
{
  fn is_subsumed(&self, store: &VStore) -> Trilean {
    !XEqY::new(self.x.bclone(), self.y.bclone()).is_subsumed(store)
  }
}

impl<VStore, Dom, Bound> Propagator<VStore> for XNeqY<VStore> where
  VStore: Collection<Item=Dom>,
  Dom: Bounded<Item=Bound> + Cardinality + Difference<Bound, Output=Dom>,
  Bound: PartialOrd
{
  fn propagate(&mut self, store: &mut VStore) -> bool {
    let x = self.x.read(store);
    let y = self.y.read(store);

    if x.is_singleton() {
      self.y.update(store, y.difference(&x.lower()))
    }
    else if y.is_singleton() {
      self.x.update(store, x.difference(&y.lower()))
    }
    else {
      true
    }
  }
}

impl<VStore> PropagatorDependencies<FDEvent> for XNeqY<VStore> where
 VStore: Collection,
 XEqY<VStore>: PropagatorDependencies<FDEvent>
{
  fn dependencies(&self) -> Vec<(usize, FDEvent)> {
    XEqY::new(self.x.bclone(), self.y.bclone()).dependencies()
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use kernel::Trilean::*;
  use propagation::events::FDEvent::*;
  use interval::interval::*;
  use propagators::test::*;

  #[test]
  fn x_neq_y_test() {
    let dom0_10 = (0,10).to_interval();
    let dom10_20 = (10,20).to_interval();
    let dom5_15 = (5,15).to_interval();
    let dom11_20 = (11,20).to_interval();
    let one = (1,1).to_interval();
    let zero = (0,0).to_interval();
    let ten = (10,10).to_interval();

    x_neq_y_test_one(1, dom0_10, dom0_10, Unknown, Unknown, vec![], true);
    x_neq_y_test_one(2, dom0_10, dom10_20, Unknown, Unknown, vec![], true);
    x_neq_y_test_one(3, dom5_15, dom10_20, Unknown, Unknown, vec![], true);
    x_neq_y_test_one(4, dom0_10, dom11_20, True, True, vec![], true);
    x_neq_y_test_one(5, one, dom0_10, Unknown, Unknown, vec![], true);
    x_neq_y_test_one(6, zero, dom0_10, Unknown, True, vec![(1, Bound)], true);
    x_neq_y_test_one(7, ten, dom0_10, Unknown, True, vec![(1, Bound)], true);
    x_neq_y_test_one(8, one, one, False, False, vec![], false);
    x_neq_y_test_one(9, zero, one, True, True, vec![], true);
  }

  fn x_neq_y_test_one(test_num: u32, x: Interval<i32>, y: Interval<i32>,
    before: Trilean, after: Trilean,
    delta_expected: Vec<(usize, FDEvent)>, propagate_success: bool)
  {
    binary_propagator_test(test_num, XNeqY::new, x, y, before, after, delta_expected, propagate_success);
  }
}
