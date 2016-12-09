// Copyright 2016 Pierre Talbot (IRCAM)

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
use propagators::PropagatorKind;
use propagators::cmp::{XGreaterEqYPlusZ, XLessEqYPlusZ, x_geq_y_plus_z, x_leq_y_plus_z};
use propagation::*;
use propagation::events::*;
use std::fmt::{Formatter, Debug, Error};
use num::{Signed, PrimInt};

#[derive(Clone, Copy)]
pub struct XEqYPlusZ<X, Y, Z, B>
{
  geq: XGreaterEqYPlusZ<X, Y, Z, B>,
  leq: XLessEqYPlusZ<X, Y, Z, B>
}

impl<X, Y, Z, B> PropagatorKind for XEqYPlusZ<X, Y, Z, B> {}

impl<X, Y, Z, B> XEqYPlusZ<X, Y, Z, B> where
  X: Clone,
  Y: Clone,
  Z: Clone,
  B: PrimInt + Signed,
{
  pub fn new(x: X, y: Y, z: Z) -> XEqYPlusZ<X, Y, Z, B> {
    XEqYPlusZ {
      geq: x_geq_y_plus_z(x.clone(), y.clone(), z.clone()),
      leq: x_leq_y_plus_z(x, y, z)
    }
  }
}

impl<X, Y, Z, B> Debug for XEqYPlusZ<X, Y, Z, B> where
  X: Debug,
  Y: Debug,
  Z: Debug,
  B: Debug
{
  fn fmt(&self, formatter: &mut Formatter) -> Result<(), Error> {
    formatter.write_fmt(format_args!(
      "X = Y + Z (decomposed into: {:?} and {:?}).",
      self.geq, self.leq))
  }
}

impl<Store, X, Y, Z, B> Subsumption<Store> for XEqYPlusZ<X, Y, Z, B> where
  XGreaterEqYPlusZ<X, Y, Z, B>: Subsumption<Store>,
  XLessEqYPlusZ<X, Y, Z, B>: Subsumption<Store>,
{
  fn is_subsumed(&self, store: &Store) -> Trilean {
    self.geq.is_subsumed(store).and(self.leq.is_subsumed(store))
  }
}

impl<Store, X, Y, Z, B> Propagator<Store> for XEqYPlusZ<X, Y, Z, B> where
  XGreaterEqYPlusZ<X, Y, Z, B>: Propagator<Store>,
  XLessEqYPlusZ<X, Y, Z, B>: Propagator<Store>
{
  fn propagate(&mut self, store: &mut Store) -> bool {
    self.geq.propagate(store) &&
    self.leq.propagate(store)
  }
}

impl<X, Y, Z, B> PropagatorDependencies<FDEvent> for XEqYPlusZ<X, Y, Z, B> where
  XGreaterEqYPlusZ<X, Y, Z, B>: PropagatorDependencies<FDEvent>,
  XLessEqYPlusZ<X, Y, Z, B>: PropagatorDependencies<FDEvent>
{
  fn dependencies(&self) -> Vec<(usize, FDEvent)> {
    let geq_deps = self.geq.dependencies();
    let leq_deps = self.leq.dependencies();
    assert_eq!(geq_deps, leq_deps,
      "This function assumed both dependencies of X >= Y + Z and X <= Y + Z are equals.");
    geq_deps
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
  fn x_eq_y_plus_z_test() {
    let dom0_10 = (0,10).to_interval();
    let dom10_20 = (10,20).to_interval();
    let dom12_12 = (12,12).to_interval();
    let dom0_6 = (0,6).to_interval();
    let dom0_5 = (0,5).to_interval();
    let dom0_1 = (0,1).to_interval();
    let dom1_1 = (1,1).to_interval();
    let dom2_2 = (2,2).to_interval();

    x_eq_y_plus_z_test_one(1, dom0_10, dom0_10, dom0_10, Unknown, Unknown, vec![], true);
    x_eq_y_plus_z_test_one(2, dom12_12, dom0_6, dom0_6, Unknown, True, vec![(1, Assignment), (2, Assignment)], true);
    x_eq_y_plus_z_test_one(3, dom10_20, dom1_1, dom1_1, False, False, vec![], false);
    x_eq_y_plus_z_test_one(4, dom2_2, dom1_1, dom1_1, True, True, vec![], true);
    x_eq_y_plus_z_test_one(5, dom1_1, dom2_2, dom2_2, False, False, vec![], false);
    x_eq_y_plus_z_test_one(6, dom0_6, dom0_5, dom0_1, Unknown, Unknown, vec![], true);
  }

  fn x_eq_y_plus_z_test_one(test_num: u32,
    x: Interval<i32>, y: Interval<i32>, z: Interval<i32>,
    before: Trilean, after: Trilean,
    delta_expected: Vec<(usize, FDEvent)>, propagate_success: bool)
  {
    trinary_propagator_test(test_num, XEqYPlusZ::<_,_,_,i32>::new, x, y, z, before, after, delta_expected, propagate_success);
  }
}
