use chrono::FixedOffset;
use serde::{Serialize, Serializer};

use crate::input::{FeatureDataAndChildren, FeatureID, FeatureMap, RawFeature};

#[derive(Debug, Serialize, Clone)]
pub struct Feature {
    #[serde(rename = "feature")]
    pub id: String,
    pub progress_status: String,
    pub assigned_team: String,
    pub start: chrono::DateTime<FixedOffset>,
    pub end: chrono::DateTime<FixedOffset>,
    #[serde(serialize_with = "odered_features")]
    pub subfeatures: Vec<Feature>,
}

/// Custom serializer for [Feature]
fn odered_features<S>(value: &[Feature], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut value = value.to_owned();
    value.sort_by_key(|feature| feature.start);

    value.serialize(serializer)
}

/// Implement PartialEq so that we can compare [Feature]s in an ordered way.
impl PartialEq for Feature {
    fn eq(&self, other: &Self) -> bool {
        // sort the subfeatures by start date before equality check
        let mut these_subfeatures = self.subfeatures.clone();
        these_subfeatures.sort_by_key(|feature| feature.start);
        let mut those_subfeatures = other.subfeatures.clone();
        those_subfeatures.sort_by_key(|feature| feature.start);

        self.id == other.id
            && self.progress_status == other.progress_status
            && self.assigned_team == other.assigned_team
            && self.start == other.start
            && self.end == other.end
            && these_subfeatures == those_subfeatures
    }
}

#[derive(Debug, Serialize, PartialEq, Clone)]
pub struct Program {
    pub id: String,
    pub root: Feature,
}

#[derive(Debug, Default, Serialize, Clone)]
pub struct ProgramGraph {
    #[serde(serialize_with = "odered_programs")]
    pub programs: Vec<Program>,
}

// /// Custom serializer for features
/// Custom serializer for [Program]
fn odered_programs<S>(value: &[Program], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut value = value.to_owned();
    value.sort_by_key(|program| program.root.start);

    value.serialize(serializer)
}

/// Implement PartialEq so that we can compare [Program]s in an ordered way.
impl PartialEq for ProgramGraph {
    fn eq(&self, other: &Self) -> bool {
        // sort the programs by root start date before equality check
        let mut these_programs = self.programs.clone();
        these_programs.sort_by_key(|program| program.root.start);
        let mut those_programs = other.programs.clone();
        those_programs.sort_by_key(|program| program.root.start);

        these_programs == those_programs
    }
}

/// Transform a vector of [`RawFeature`] into a [`ProgramGraph`]
impl From<Vec<RawFeature>> for ProgramGraph {
    fn from(value: Vec<RawFeature>) -> Self {
        // given a list of child feature IDs, recursively resolve the child features
        #[tracing::instrument(skip(mappings))]
        fn resolve_subfeatures(child_ids: Vec<FeatureID>, mappings: &FeatureMap) -> Vec<Feature> {
            //tracing::debug!("resolving children: {child_ids:?}");
            mappings
                //.values()
                .iter()
                //.filter(|&value| {
                .filter(|(feature_id, value)| {
                    if let Some(feature_data) = value.feature_data {
                        child_ids.contains(&feature_data.id) // todo: use feature_id, remove if/else
                    } else {
                        tracing::debug!(feature_id, "no feature_data found");
                        false
                    }
                })
                //.filter_map(|value| {
                .filter_map(|(feature_id, value)| {
                    tracing::debug!(feature_id, "...");
                    value.feature_data.map(|feature_data| Feature {
                        id: feature_data.id.clone(),
                        start: feature_data.start_time,
                        end: feature_data.end_time,
                        assigned_team: feature_data.assigned_team,
                        progress_status: feature_data.progress_status.clone(),
                        subfeatures: resolve_subfeatures(value.children.clone(), mappings),
                    })
                })
                .collect()
        }

        // a place to hold the parent -> children references
        let mut mappings = FeatureMap::new();

        for feature in value.iter() {
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
                        feature.id.clone(),
                        FeatureDataAndChildren {
                            feature_data: Some(feature),
                            children: vec![],
                        },
                    );
                }
            };

            if let Some(parent_id) = feature.parent_id.clone() {
                match mappings.get_mut(&parent_id) {
                    Some(mapping) => mapping.children.push(feature.id.to_owned()),
                    None => {
                        mappings.insert(
                            parent_id,
                            FeatureDataAndChildren {
                                feature_data: None,
                                children: vec![feature.id.to_owned()],
                            },
                        );
                    }
                }
            };
        }

        let roots = mappings.values().filter(|&value| {
            // * is_some_and is way handier in the filter, but it's a feature only available in rust-nightly
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

        let mut graph = ProgramGraph::default();
        // this will loop over roots to build Program structs, and push them into the Graph
        for root in roots {
            if let Some(feature_data) = root.feature_data {
                let program = Program {
                    id: feature_data.program_id.clone(),
                    root: Feature {
                        id: feature_data.id.clone(),
                        start: feature_data.start_time,
                        end: feature_data.end_time,
                        assigned_team: feature_data.assigned_team,
                        progress_status: feature_data.progress_status.clone(),
                        subfeatures: resolve_subfeatures(root.children.clone(), &mappings),
                    },
                };
                graph.programs.push(program);
            } else {
                todo!("warn about missing feature_data");
            }
        }

        graph
    }
}

#[cfg(test)]
mod test {
    use crate::input::RawFeature;

    use super::{Feature, Program, ProgramGraph};
    use chrono::DateTime;

    #[test]
    fn parent_child_rawfeature_to_programgraph() {
        let input: Vec<RawFeature> = vec![
            RawFeature {
                id: "a".into(),
                parent_id: None,
                program_id: "t1".into(),
                assigned_team: "s1".into(),
                progress_status: "s1".into(),
                start_time: DateTime::parse_from_rfc3339("2023-10-01T00:00:00.000Z")
                    .expect("test dates should be checked"),
                end_time: DateTime::parse_from_rfc3339("2023-11-30T00:00:00.000Z")
                    .expect("test dates should be checked"),
            },
            RawFeature {
                id: "b".into(),
                parent_id: Some("a".into()),
                program_id: "t1".into(),
                assigned_team: "s1".into(),
                progress_status: "s2".into(),
                start_time: DateTime::parse_from_rfc3339("2023-10-20T00:00:00.000Z")
                    .expect("test dates should be checked"),
                end_time: DateTime::parse_from_rfc3339("2023-11-20T00:00:00.000Z")
                    .expect("test dates should be checked"),
            },
        ];

        let expected = ProgramGraph {
            programs: vec![Program {
                id: "t1".into(),
                root: Feature {
                    id: "a".into(),
                    progress_status: "s1".into(),
                    assigned_team: "s1".into(),
                    start: DateTime::parse_from_rfc3339("2023-10-01T00:00:00.000Z")
                        .expect("test dates should be checked"),
                    end: DateTime::parse_from_rfc3339("2023-11-30T00:00:00.000Z")
                        .expect("test dates should be checked"),
                    subfeatures: vec![Feature {
                        id: "b".into(),
                        progress_status: "s2".into(),
                        assigned_team: "s1".into(),
                        start: DateTime::parse_from_rfc3339("2023-10-20T00:00:00.000Z")
                            .expect("test dates should be checked"),
                        end: DateTime::parse_from_rfc3339("2023-11-20T00:00:00.000Z")
                            .expect("test dates should be checked"),
                        subfeatures: vec![],
                    }],
                },
            }],
        };

        assert_eq!(ProgramGraph::from(input), expected);
    }
}
