//! Class beta groups the following rules :
//! <ul>
//! <li>SCM-SCO</li>
//! <li>SCM-EQC2</li>
//! <li>SCM-SPO</li>
//! <li>SCM-EQP2</li>
//! </ul>
//!
//! All these rules have the following properties :
//! <ol>
//! <li>same predicate in both parts</li>
//! </ol>

use crate::inferray::InfGraph;
use crate::inferray::NodeDictionary;
use crate::inferray::TripleStore;

fn apply_beta_rule(graph: &InfGraph, rule_p: usize, infer_p: usize) -> TripleStore {
    let pairs = graph.dictionary.ts.elem.get(rule_p);
    if pairs == None {
        return TripleStore::new();
    }
    let infer_p = NodeDictionary::idx_to_prop_idx(infer_p);
    let pairs1 = pairs.unwrap();
    let pairs2 = pairs.unwrap();
    let mut output = TripleStore::new();
    for pair1 in &pairs1[0] {
        for pair2 in &pairs2[0] {
            if pair1[0] == pair2[1] && pair1[1] == pair2[0] {
                output.add_triple([pair1[0], infer_p, pair1[1]]);
                output.add_triple([pair2[0], infer_p, pair2[1]]);
            }
        }
    }
    output
}

fn apply_inverse_beta_rule(graph: &InfGraph, rule_p: usize, infer_p: usize) -> TripleStore {
    let pairs = graph.dictionary.ts.elem.get(rule_p);
    if pairs == None {
        return TripleStore::new();
    }
    let rule_p = NodeDictionary::idx_to_prop_idx(rule_p);
    let infer_p = NodeDictionary::idx_to_prop_idx(infer_p);
    let pairs1 = pairs.unwrap();
    let mut output = TripleStore::new();
    for pair1 in &pairs1[0] {
        output.add_triple([pair1[0], infer_p, pair1[1]]);
        output.add_triple([pair1[1], infer_p, pair1[0]]);
        output.add_triple([pair1[0], rule_p, pair1[1]]);
    }
    output
}

/// The SCM-EQC2 rule from the RDFS+ ruleset
///
/// Body:
/// - c1 rdfs:subClassOf c2
/// - c2 rdfs:subClassOf c1
///
/// Head:
/// - c1 owl:equivalentClass c2
/// - c2 owl:equivalentClass c1
pub fn SCM_EQC2(graph: &mut InfGraph) -> TripleStore {
    let id_1 = NodeDictionary::prop_idx_to_idx(graph.dictionary.rdfssubClassOf as u64);
    let id_2 = NodeDictionary::prop_idx_to_idx(graph.dictionary.owlequivalentClass as u64);
    apply_beta_rule(graph, id_1, id_2)
}

pub fn SCM_EQP2(graph: &mut InfGraph) -> TripleStore {
    let id_1 = NodeDictionary::prop_idx_to_idx(graph.dictionary.rdfssubPropertyOf as u64);
    let id_2 = NodeDictionary::prop_idx_to_idx(graph.dictionary.owlequivalentProperty as u64);
    apply_beta_rule(graph, id_1, id_2)
}

pub fn SCM_EQC1(graph: &mut InfGraph) -> TripleStore {
    let id_1 = NodeDictionary::prop_idx_to_idx(graph.dictionary.owlequivalentClass as u64);
    let id_2 = NodeDictionary::prop_idx_to_idx(graph.dictionary.rdfssubClassOf as u64);
    apply_inverse_beta_rule(graph, id_1, id_2)
}

pub fn SCM_EQP1(graph: &mut InfGraph) -> TripleStore {
    let id_1 = NodeDictionary::prop_idx_to_idx(graph.dictionary.owlequivalentProperty as u64);
    let id_2 = NodeDictionary::prop_idx_to_idx(graph.dictionary.rdfssubPropertyOf as u64);
    apply_inverse_beta_rule(graph, id_1, id_2)
}
