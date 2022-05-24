// Copyright 2022 Singularity Data
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::fmt;

use risingwave_common::error::Result;
use risingwave_pb::batch_plan::plan_node::NodeBody;
use risingwave_pb::batch_plan::SortAggNode;

use super::logical_agg::PlanAggCall;
use super::{LogicalAgg, PlanBase, PlanRef, PlanTreeNodeUnary, ToBatchProst, ToDistributedBatch};
use crate::optimizer::plan_node::{BatchExchange, ToLocalBatch};
use crate::optimizer::property::{Distribution, Order, RequiredDist};

#[derive(Debug, Clone)]
pub struct BatchSimpleAgg {
    pub base: PlanBase,
    logical: LogicalAgg,
}

impl BatchSimpleAgg {
    pub fn new(logical: LogicalAgg) -> Self {
        let ctx = logical.base.ctx.clone();
        let input = logical.input();
        let input_dist = input.distribution();
        let dist = match input_dist {
            Distribution::Single => Distribution::Single,
            // distribution phase will perform total agg for this.
            Distribution::SomeShard => Distribution::SomeShard,
            _ => panic!(),
        };
        let base = PlanBase::new_batch(ctx, logical.schema().clone(), dist, Order::any().clone());
        BatchSimpleAgg { base, logical }
    }

    pub fn agg_calls(&self) -> &[PlanAggCall] {
        self.logical.agg_calls()
    }
}

impl fmt::Display for BatchSimpleAgg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("BatchSimpleAgg")
            .field("aggs", &self.agg_calls())
            .finish()
    }
}

impl PlanTreeNodeUnary for BatchSimpleAgg {
    fn input(&self) -> PlanRef {
        self.logical.input()
    }

    fn clone_with_input(&self, input: PlanRef) -> Self {
        Self::new(self.logical.clone_with_input(input))
    }
}
impl_plan_tree_node_for_unary! { BatchSimpleAgg }

impl ToDistributedBatch for BatchSimpleAgg {
    fn to_distributed(&self) -> Result<PlanRef> {
        // Ensure input is distributed, batch phase might not distribute it
        // (e.g. BatchSeqScan::{new, to_distributed})
        let dist_input = self.input().to_distributed()?;
        let plan = match dist_input.distribution() {
            Distribution::SomeShard => {
                // 2-phase agg
                let partial_agg = dist_input.clone();
                let partial_agg = self.clone_with_input(partial_agg).into();

                let exchange = BatchExchange::new(
                    partial_agg,
                    Order::any().clone(),
                    Distribution::Single
                ).into();

                let total_agg_types = self
                    .logical
                    .agg_calls()
                    .iter()
                    .map(|agg_call| agg_call.partial_to_total_agg_call())
                    .collect();
                let total_agg_logical = LogicalAgg::new(
                    total_agg_types,
                    self.logical.group_keys().to_vec(),
                    exchange
                );
                let total_agg_batch = BatchSimpleAgg::new(total_agg_logical);
                // &RequiredDist::PhysicalDist(Distribution::Single).enforce(total_agg_batch, &Order::any())
                total_agg_batch.into()
            }
            _ => {
                let new_input = self
                    .input()
                    .to_distributed_with_required(Order::any(), &RequiredDist::single())?;
                self.clone_with_input(new_input).into()
            }
        };
        Ok(plan)
    }
}

impl ToBatchProst for BatchSimpleAgg {
    fn to_batch_prost_body(&self) -> NodeBody {
        NodeBody::SortAgg(SortAggNode {
            agg_calls: self
                .agg_calls()
                .iter()
                .map(PlanAggCall::to_protobuf)
                .collect(),
            // We treat simple agg as a special sort agg without group keys.
            group_keys: vec![],
        })
    }
}

impl ToLocalBatch for BatchSimpleAgg {
    fn to_local(&self) -> Result<PlanRef> {
        let new_input = self.input().to_local()?;

        let new_input = RequiredDist::single().enforce_if_not_satisfies(new_input, Order::any())?;

        Ok(self.clone_with_input(new_input).into())
    }
}
