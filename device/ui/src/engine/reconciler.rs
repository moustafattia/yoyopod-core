use crate::animation::TimelineSampler;

use super::{Element, Mutation};

#[derive(Debug, Default)]
pub struct Reconciler;

impl Reconciler {
    pub fn diff(
        &mut self,
        _previous: Option<&Element>,
        _next: &Element,
        _sampler: &TimelineSampler<'_>,
        out: &mut Vec<Mutation>,
    ) {
        out.clear();
    }
}
