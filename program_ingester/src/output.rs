/// This needs documentation then I can write the main story
use chrono::FixedOffset;
use serde::{Serialize, Serializer};

use crate::input::{FeatureDataAndChildren, FeatureID, FeatureMap, RawFeature};


/// The lines #[derive(Debug, Serialize, Clone)] use Rust's "derive" macro to automatically generate implementations for the "Debug",
/// "Serialize", and "Clone" traits for the Feature struct. This means that instances of Feature can be debugged, serialized
/// (converted to a format like JSON or BSON), and cloned (duplicated).
// /The line #[serde(rename = "feature")] uses Serde's "serde" attribute to
/// specify that the "id" field should be renamed to "feature" when serializing or deserializing instances of Feature.
/// The line #[serde(serialize_with = "odered_features")] uses Serde's "serde" attribute to specify that the "subfeatures"
/// field should be serialized using a custom serialization function named "odered_features". This allows for custom logic
/// to be used when serializing the subfeatures field.
#[derive(Debug, Serialize, Clone)]
pub struct Feature {
    #[serde(rename = "feature")]
    pub id: String,
    pub progress_status: String,
    pub assigned_team: String,
    pub start_date: chrono::DateTime<FixedOffset>,
    pub end_date: chrono::DateTime<FixedOffset>,
    #[serde(serialize_with = "odered_features")]
    pub subfeatures: Vec<Feature>,
}


/// The given code defines a function named "odered_features", which is a custom serializer for instances of the Feature struct.
/// The function takes in a slice of Feature objects, "value", and a Serde serializer, "serializer".
/// The function sorts the input slice of Feature objects by the "start_date" field and then serializes the sorted slice using the provided serializer.
/// The function returns a Result type that represents the outcome of the serialization process.
/// If the serialization is successful, the result will contain the serialized value of type S::Ok.
/// If an error occurs during serialization, the result will contain an error value of type S::Error.
/// The function uses the "where" clause to specify that the type of the serializer must implement the "Serializer" trait.
/// This allows the function to be used with any serializer that implements this trait, making it more flexible and reusable.
/// The method to_owned() is a method that creates a new owned value (a deep copy) from a borrowed value, such as a reference.
/// In this case, the method is used to convert the input slice of Feature objects, "value", into an owned vector of Feature objects.
/// This is necessary because the sorting operation needs to modify the contents of the vector, and a reference to the original slice cannot be modified.
/// By creating an owned copy, the original data remains unchanged, and the sort operation can be performed on the copy.

//// Custom serializer for [Feature]
fn odered_features<S>(value: &[Feature], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut value = value.to_owned();
    value.sort_by_key(|feature| feature.start_date);

    value.serialize(serializer)
}

/// Implement PartialEq so that we can compare [Feature]s in an ordered way.
impl PartialEq for Feature {
    fn eq(&self, other: &Self) -> bool {
        // sort the subfeatures by start_date date before equality check
        let mut these_subfeatures = self.subfeatures.clone();
        these_subfeatures.sort_by_key(|feature| feature.start_date);
        let mut those_subfeatures = other.subfeatures.clone();
        those_subfeatures.sort_by_key(|feature| feature.start_date);

        self.id == other.id
            && self.progress_status == other.progress_status
            && self.assigned_team == other.assigned_team
            && self.start_date == other.start_date
            && self.end_date == other.end_date
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
    value.sort_by_key(|program| program.root.start_date);

    value.serialize(serializer)
}

/// Implement PartialEq so that we can compare [Program]s in an ordered way.
impl PartialEq for ProgramGraph {
    fn eq(&self, other: &Self) -> bool {
        // sort the programs by root start_date date before equality check
        let mut these_programs = self.programs.clone();
        these_programs.sort_by_key(|program| program.root.start_date);
        let mut those_programs = other.programs.clone();
        those_programs.sort_by_key(|program| program.root.start_date);

        these_programs == those_programs
    }
}


/// This code takes a vector of RawFeature objects and transforms it into a ProgramGraph object.
/// It does so by first creating a mapping of parent to children feature IDs
/// and then using this mapping to resolve the subfeatures of each feature.

/// The first step in the transformation is to build the mapping by upserting
/// each feature and its parent. If a feature or its parent already exists in the mapping,
/// it is updated, otherwise it is inserted with the given information.

/// Once the mapping is built, the code filters out the root features,
/// which are the features that don't have a parent. For each root feature,
/// the code creates a Program object by resolving its subfeatures and then pushes it into the ProgramGraph object.

/// The code uses a helper function, resolve_subfeatures,
/// to resolve the subfeatures of a feature. It takes a list of child feature
/// IDs and the mapping and returns a list of Feature objects by filtering the mapping and transforming the filtered values into Feature objects.


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
                        start_date: feature_data.start_date,
                        end_date: feature_data.end_date,
                        assigned_team: feature_data.assigned_team.clone(),
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
            // By the end_date, there should be no feature_data values set to None

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
                        start_date: feature_data.start_date,
                        end_date: feature_data.end_date,
                        assigned_team: feature_data.assigned_team.clone(),
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
                start_date: DateTime::parse_from_rfc3339("2023-10-01T00:00:00.000Z")
                    .expect("test dates should be checked"),
                end_date: DateTime::parse_from_rfc3339("2023-11-30T00:00:00.000Z")
                    .expect("test dates should be checked"),
            },
            RawFeature {
                id: "b".into(),
                parent_id: Some("a".into()),
                program_id: "t1".into(),
                assigned_team: "s1".into(),
                progress_status: "s2".into(),
                start_date: DateTime::parse_from_rfc3339("2023-10-20T00:00:00.000Z")
                    .expect("test dates should be checked"),
                end_date: DateTime::parse_from_rfc3339("2023-11-20T00:00:00.000Z")
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
                    start_date: DateTime::parse_from_rfc3339("2023-10-01T00:00:00.000Z")
                        .expect("test dates should be checked"),
                    end_date: DateTime::parse_from_rfc3339("2023-11-30T00:00:00.000Z")
                        .expect("test dates should be checked"),
                    subfeatures: vec![Feature {
                        id: "b".into(),
                        progress_status: "s2".into(),
                        assigned_team: "s1".into(),
                        start_date: DateTime::parse_from_rfc3339("2023-10-20T00:00:00.000Z")
                            .expect("test dates should be checked"),
                        end_date: DateTime::parse_from_rfc3339("2023-11-20T00:00:00.000Z")
                            .expect("test dates should be checked"),
                        subfeatures: vec![],
                    }],
                },
            }],
        };

        assert_eq!(ProgramGraph::from(input), expected);
    }
}
