use crate::inferray::NodeDictionary;
use crate::inferray::TripleStore;

use rayon::prelude::*;

#[cfg_attr(debug_assertions, flamer::flame)]
fn apply_delta_rule(
    ts: &TripleStore,
    prop_idx: usize,
    invert: bool,
) -> Box<dyn Iterator<Item = [u64; 3]>> {
    let mut output = vec![];
    if let Some(pairs) = ts.elem().get(prop_idx) {
        for pair in pairs.so() {
            if pair[0] != pair[1] {
                let prop_idx = NodeDictionary::prop_idx_to_idx(pair[0]);
                if let Some(usable_pairs) = ts.elem().get(prop_idx) {
                    let usable_pairs = if invert {
                        usable_pairs.os()
                    } else {
                        usable_pairs.so()
                    };
                    for usable_pair in usable_pairs {
                        output.push([usable_pair[0], pair[1], usable_pair[1]]);
                    }
                }
                let prop_idx = NodeDictionary::prop_idx_to_idx(pair[1]);
                if let Some(usable_pairs) = ts.elem().get(prop_idx) {
                    let usable_pairs = if invert {
                        usable_pairs.os()
                    } else {
                        usable_pairs.so()
                    };
                    for usable_pair in usable_pairs {
                        output.push([usable_pair[0], pair[0], usable_pair[1]]);
                    }
                }
            }
        }
    }
    Box::new(output.into_iter())
}

#[cfg_attr(debug_assertions, flamer::flame)]
pub fn PRP_INV_1_2(ts: &TripleStore) -> Box<dyn Iterator<Item = [u64; 3]>> {
    apply_delta_rule(
        ts,
        NodeDictionary::prop_idx_to_idx(NodeDictionary::owlinverseOf as u64),
        true,
    )
}

#[cfg_attr(debug_assertions, flamer::flame)]
pub fn PRP_EQP_1_2(ts: &TripleStore) -> Box<dyn Iterator<Item = [u64; 3]>> {
    apply_delta_rule(
        ts,
        NodeDictionary::prop_idx_to_idx(NodeDictionary::owlequivalentProperty as u64),
        false,
    )
}
