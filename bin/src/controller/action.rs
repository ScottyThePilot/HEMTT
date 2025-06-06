use hemtt_common::arma::control::{fromarma, toarma};

use crate::context::Context;

use super::profile::AutotestMission;

pub trait Action {
    fn missions(&self, ctx: &Context) -> Vec<(String, AutotestMission)>;
    fn incoming(&self, ctx: &Context, msg: fromarma::Message) -> Vec<toarma::Message>;
    fn outgoing(&self, _ctx: &Context) -> Vec<toarma::Message> {
        Vec::new()
    }
}
