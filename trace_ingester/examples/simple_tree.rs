use std::{collections::HashMap, fmt::Display};

use serde::Serialize;

// todo: we would implement Deserialize on this to be able to parse in the serialised form. Eg: 1->null or 2->1
#[derive(Debug)]
pub struct RawFeature {
    /// This node's ID
    id: u8,

    /// If None, then it's a root
    parent_id: Option<u8>,

    /// program id
    program_id: u8,
    // todo: other fields
}

impl RawFeature {
    fn is_root(&self) -> bool {
        self.parent_id.is_none()
    }
}

/// Augment the output of the root with a `*`
impl Display for RawFeature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.is_root() {
            true => write!(f, "*{}", self.id),
            false => write!(f, "{}", self.id),
        }
    }
}

#[derive(Default, Debug, Serialize)]
pub struct Feature {
    id: u8,
    subfeatures: Vec<Feature>,
}

#[derive(Default, Debug, Serialize)]
pub struct Program {
    id: u8,
    root: Feature,
}

#[derive(Default, Debug, Serialize)]
pub struct Graph {
    programs: Vec<Program>,
}

// I avoided using a Tuple as the value of the hashmap as it doesn't communicate intent
#[derive(Debug)]
struct FeatureDataAndChildren<'a> {
    // This refers to existing data already ingested
    feature_data: Option<&'a RawFeature>,

    // These are the child feature IDs which can then be looked up in the FeatureMap (HashMap)
    children: Vec<u8>,
}
// alias types to make usage simpler and more expressive
type FeatureID = u8;
type FeatureMap<'a> = HashMap<FeatureID, FeatureDataAndChildren<'a>>;

fn main() {
    // We assume that the events are coming in out of order from distributed systems
    let sequence = vec![
        &RawFeature {
            id: 3,
            parent_id: Some(2),
            program_id: 1,
        },
        &RawFeature {
            id: 1,
            parent_id: None,
            program_id: 1,
        },
        &RawFeature {
            id: 2,
            parent_id: Some(1),
            program_id: 1,
        },
        &RawFeature {
            id: 4,
            parent_id: Some(1),
            program_id: 1,
        },
    ];

    // a place to hold the parent -> children references
    let mut mappings = FeatureMap::new();

    // print sequence
    print!("Input Sequence:");
    for feature in sequence {
        print!(" {feature}");
        //println!("{feature:#?}");

        // 1. Upsert this feature in the mappings:
        //    - If it exists already (a child created it to add itself to children, see #2): Set the feature_data to Some(...)
        //    - If it doesn't exist: Insert a new entry with the feature_data set to Some(...) and the children as an empty Vec
        // 2. Upsert the parent feature in the mappings
        //    - If it exists already (created by #1, or by another child feature in #2): Only push this id as a child.
        //    - If it doesn't exist: Insert a new entry with the feature_data set to None and the children as a Vec with one entry (this feature id)
        // By the end, there should be no feature_data values set to None

        match mappings.get_mut(&feature.id) {
            Some(mapping) => mapping.feature_data = Some(feature),
            None => {
                mappings.insert(
                    feature.id,
                    FeatureDataAndChildren {
                        feature_data: Some(feature),
                        children: vec![],
                    },
                );
            }
        }

        if let Some(parent_id) = feature.parent_id {
            match mappings.get_mut(&parent_id) {
                Some(mapping) => mapping.children.push(feature.id),
                None => {
                    mappings.insert(
                        parent_id,
                        FeatureDataAndChildren {
                            feature_data: None,
                            children: vec![feature.id],
                        },
                    );
                }
            }
        }
    }

    println!();
    println!("{mappings:#?}");
    println!("\n#######################################\n");

    let roots = mappings
        .values() //
        .filter(|&value| {
            // // is_some_and is way handier in the filter, but it's a feature only available in rust-nightly
            // value
            //     .feature_data
            //     .is_some_and(|&feature_data| feature_data.is_root())
            // But alas, we have to do it the uglier way for now:
            if let Some(feature_data) = value.feature_data {
                feature_data.is_root()
            } else {
                false
            }
        });

    let mut graph = Graph::default();
    // this will loop over roots to build Program structs, and push them into the Graph
    for root in roots {
        println!("{root:#?}");

        if let Some(feature_data) = root.feature_data {
            let program = Program {
                id: feature_data.program_id,
                root: Feature {
                    id: feature_data.id,
                    subfeatures: resolve_subfeatures(root.children.clone(), &mappings),
                },
            };
            graph.programs.push(program);
        } else {
            todo!("warn about missing feature_data");
        }
    }

    println!("\n#######################################\n");

    //println!("{graph:#?}");
    let json = serde_json::to_string(&graph).unwrap();
    println!("{json}");
}

// given a list of child feature IDs, recursively resolve the child features
fn resolve_subfeatures(child_ids: Vec<u8>, mappings: &FeatureMap) -> Vec<Feature> {
    mappings
        .values()
        .filter(|&value| {
            if let Some(feature_data) = value.feature_data {
                child_ids.contains(&feature_data.id)
            } else {
                false
            }
        })
        .filter_map(|value| {
            value.feature_data.map(|feature_data| Feature {
                id: feature_data.id,
                subfeatures: resolve_subfeatures(value.children.clone(), mappings),
            })
        })
        .collect()
}
