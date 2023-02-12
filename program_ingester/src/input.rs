/// A crate for defining the input structs and implement their traits
use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Read},
    str::FromStr,
};

use chrono::{DateTime, FixedOffset};

use crate::errors::ProgramIngesterError;

/// This code defines a struct named Ingester and its implementation of the TryFrom trait.
/// The struct Ingester has one public field, features, which is a vector of RawFeatures.
/// The TryFrom trait is a standard trait in Rust and is used to define conversions from one type to another.
/// In this case, the TryFrom implementation for Ingester takes a BufReader<R> as input, where R is a type that implements the Read trait.
/// The BufReader<R> is a type that wraps a Read and provides a convenient interface for reading data.
/// The implementation of TryFrom for Ingester reads lines from the BufReader<R> using the read_line method, which reads until it reaches a newline character.
/// For each line read, it removes the trailing newline character using trim_end, converts the line to a RawFeature using the from_str method, and then pushes the RawFeature to the features vector.
/// Finally, the implementation returns an Ok variant of a Result containing the Ingester struct, with its features field populated with the RawFeatures created from the input data.
/// If any errors occur during the reading or conversion process, the implementation returns an Err variant of the Result,
/// with the error being of type crate::errors::ProgramIngesterError.

/// The main entrypoint
pub struct Ingester {
    pub features: Vec<RawFeature>,
}

// // Todo: make this work to produce a vec of features from a reader (eg: file, stdin, etc...)
impl<R: Read> TryFrom<BufReader<R>> for Ingester {
    type Error = crate::errors::ProgramIngesterError;

    fn try_from(mut reader: BufReader<R>) -> Result<Self, Self::Error> {
        let mut features = vec![];
        let mut buf = String::new();

        // It's more efficient to allocate a single string buffer and loop
        // over reader.read_line(), rather than using reader.lines().map()
        // which will allocate a new String on each iteration
        while reader.read_line(&mut buf)? > 0 {
            {
                // remove the trailing \n
                let line = buf.trim_end();
                let feature = RawFeature::from_str(line)?;
                features.push(feature);
            }
            buf.clear();
        }
        Ok(Ingester { features })
    }
}

/// RawFeature represents a struct that contains 7 fields.
/// There are implementations for the TryFrom and FromStr traits for the RawFeature struct.
/// The TryFrom implementation allows creating a RawFeature instance from a String,
/// and the FromStr implementation allows creating a RawFeature instance from a string slice (&str).
/// The FromStr implementation tries to turn a program log line into a feature by parsing the string slice
/// into its component parts. If the string slice doesn't have the expected format or can't be parsed into a RawFeature,
/// then the implementation returns an error.

#[derive(Debug, PartialEq, Eq)]
pub struct RawFeature {
    /// This node's ID
    pub id: String,

    /// The parent feature
    ///
    /// If it is set to `None`, then this is a root feature
    pub parent_id: Option<String>,

    /// Program ID
    pub program_id: String,

    /// progress_status (Complete, In Progress)
    pub progress_status: String,

    /// The name of the assigned team generating the feature
    pub assigned_team: String,

    /// The Feature Start Time
    pub start_date: chrono::DateTime<FixedOffset>,

    /// Feature End Time
    ///
    pub end_date: chrono::DateTime<FixedOffset>,
}

impl RawFeature {
    pub fn is_root(&self) -> bool {
        self.parent_id.is_none()
    }
}

impl TryFrom<String> for RawFeature {
    type Error = crate::errors::ProgramIngesterError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        RawFeature::from_str(value.as_str())
    }
}

impl FromStr for RawFeature {
    type Err = crate::errors::ProgramIngesterError;

    /// Try to turn a program log line into a feature
    ///
    /// Example: `2016-10-20T12:43:34.000Z 2016-10-20T12:43:35.000Z program1 back-end-3 ac->ad`
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.trim().split(" ").collect();
        // We should have 6 parts.
        if parts.len() != 6 {
            return Err(ProgramIngesterError::InvalidProgramInput(format!(
                "The feature '{s}' needs to have 6 parts, start, end, program, progress_status, assigned_team, feature-relation"
            )));
        }

        // If so, then we need to split the last one part
        let last_part = parts
            .last()
            .expect("we checked that there are 6 parts earlier");
        let feature_ids: Vec<&str> = last_part.split("->").collect();

        if feature_ids.len() != 2 {
            return Err(ProgramIngesterError::InvalidProgramInput(format!(
                "The feature-relation '{s}' needs to have 2 parts"
            )));
        }

        Ok(RawFeature {
            id: feature_ids.last().unwrap().to_owned().into(),
            parent_id: match feature_ids.first().unwrap().to_owned() {
                "null" => None,
                id => Some(id.into()),
            },
            start_date: DateTime::parse_from_rfc3339(parts.first().unwrap().to_owned())?,
            end_date: DateTime::parse_from_rfc3339(parts.get(1).unwrap().to_owned())?,
            program_id: parts.get(2).unwrap().to_owned().into(),
            progress_status: parts.get(3).unwrap().to_owned().into(),
            assigned_team: parts.get(4).unwrap().to_owned().into(),
        })
    }
}

/// The FeatureDataAndChildren struct is a struct that holds two pieces of data:
/// feature_data, which is an Option of a reference to a RawFeature struct.
/// This means that feature_data could either contain a reference to a RawFeature instance,
/// or it could be None, indicating that there is no RawFeature instance associated with this struct.
/// children, which is a Vec (vector) of FeatureID values. This is a list of child feature
/// IDs that can be used to look up related information in the FeatureMap.
/// FeatureMap, is a type alias for a HashMap (hash map) data structure.
/// A HashMap is a collection of key-value pairs, where the keys are of type FeatureID
/// (which is defined as a type alias for String) and the values are of type FeatureDataAndChildren.
/// This type alias makes it more expressive and easier to use, as the type FeatureMap is more
/// meaningful and understandable than the underlying type HashMap.
/// 'a is a lifetime annotation. In Rust, lifetimes are a way of expressing the relationship between references.
/// They ensure that references are used in a safe and correct way by preventing references from pointing to data that has been dropped.

#[derive(Debug)]
pub struct FeatureDataAndChildren<'a> {
    // This refers to existing data already ingested
    pub feature_data: Option<&'a RawFeature>,

    // These are the child feature IDs which can then be looked up in the FeatureMap (HashMap)
    pub children: Vec<FeatureID>,
}
// alias types to make usage simpler and more expressive
pub type FeatureID = String;
pub type FeatureMap<'a> = HashMap<FeatureID, FeatureDataAndChildren<'a>>;

#[cfg(test)]
mod test {
    use chrono::DateTime;
    use std::str::FromStr;

    use super::RawFeature;

    #[test]
    fn test_parsing_single_program() {
        let input = "2023-01-01T00:00:00.000Z 2023-06-30T00:00:00.000Z program1 Complete TeamB ProductivitySuite->Email";

        let expected = RawFeature {
            id: "Email".into(),
            parent_id: Some("ProductivitySuite".into()),
            program_id: "program1".into(),
            progress_status: "Complete".into(),
            assigned_team: "TeamB".into(),
            start_date: DateTime::parse_from_rfc3339("2023-01-01T00:00:00.000Z")
                .expect("test dates should be checked"),
            end_date: DateTime::parse_from_rfc3339("2023-06-30T00:00:00.000Z")
                .expect("test dates should be checked"),
        };

        let actual = RawFeature::from_str(input);
        assert!(actual.is_ok());
        if let Ok(parsed) = actual {
            assert_eq!(parsed, expected);
        }
    }
}
