use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Read},
    str::FromStr,
};

use chrono::{DateTime, FixedOffset};

use crate::errors::ProgramInjestorError;

/// The main entrypoint
pub struct Ingester {
    pub features: Vec<RawFeature>,
}

// // Todo: make this work to produce a vec of features from a reader (eg: file, stdin, etc...)
impl<R: Read> TryFrom<BufReader<R>> for Ingester {
    type Error = crate::errors::ProgramInjestorError;

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

#[derive(Debug, PartialEq, Eq)]
pub struct RawFeature {
    /// This node's ID
    pub id: String,

    /// The parent feature
    ///
    /// If it is set to `None`, then this is a root feature
    pub parent_id: Option<String>,

    /// Program ID
    ///
    /// A number of related features form a program
    pub program_id: String,

    /// The name of the progress_status generating the feature
    pub progress_status: String,

    /// The name of the progress_status generating the feature
    pub assigned_team: String,

    /// The Feature Start Time
    pub start_time: chrono::DateTime<FixedOffset>,

    /// Feature End Time
    ///
    /// **Note**: This could be later than child features if the child features are asynchronous.
    pub end_time: chrono::DateTime<FixedOffset>,
}

impl RawFeature {
    pub fn is_root(&self) -> bool {
        self.parent_id.is_none()
    }
}

impl TryFrom<String> for RawFeature {
    type Error = crate::errors::ProgramInjestorError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        RawFeature::from_str(value.as_str())
    }
}

impl FromStr for RawFeature {
    type Err = crate::errors::ProgramInjestorError;

    /// Try to turn a program log line into a feature
    ///
    /// Example: `2016-10-20T12:43:34.000Z 2016-10-20T12:43:35.000Z program1 back-end-3 ac->ad`
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.trim().split(" ").collect();
        // We should have 6 parts.
        if parts.len() != 6 {
            return Err(ProgramInjestorError::InvalidProgramInput(format!(
                "The feature '{s}' needs to have 5 parts, start, end, program, progress_status, assigned_team, feature-relation"
            )));
        }

        // If so, then we need to split the last one part
        let last_part = parts
            .last()
            .expect("we checked that there are 5 parts earlier");
        let feature_ids: Vec<&str> = last_part.split("->").collect();

        if feature_ids.len() != 2 {
            return Err(ProgramInjestorError::InvalidProgramInput(format!("The feature-relation '{s}' needs to have 5 parts, start, end, program, progress_status, feature-relation")));
        }

        Ok(RawFeature {
            id: feature_ids.last().unwrap().to_owned().into(),
            parent_id: match feature_ids.first().unwrap().to_owned() {
                "null" => None,
                id => Some(id.into()),
            },
            start_time: DateTime::parse_from_rfc3339(parts.first().unwrap().to_owned())?,
            end_time: DateTime::parse_from_rfc3339(parts.get(1).unwrap().to_owned())?,
            program_id: parts.get(2).unwrap().to_owned().into(),
            progress_status: parts.get(3).unwrap().to_owned().into(),
            assigned_team: parts.get(index: 4).unwrap().to_owned().into(),
        })
    }
}

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
        let input = "2023-01-01T00:00:00.000Z 2023-06-30T00:00:00.000Z program_1 Complete Team_B Productivity_Suite->Email";

        let expected = RawFeature {
            id: "Email".into(),
            parent_id: Some("Productivity_Suite".into()),
            program_id: "program1".into(),
            progress_status: "Complete".into(),
            assigned_team: "Team_B".into(),
            start_time: DateTime::parse_from_rfc3339("2023-01-01T00:00:00.000Z")
                .expect("test dates should be checked"),
            end_time: DateTime::parse_from_rfc3339("2023-06-30T00:00:00.000Z")
                .expect("test dates should be checked"),
        };

        let actual = RawFeature::from_str(input);
        assert!(actual.is_ok());
        if let Ok(parsed) = actual {
            assert_eq!(parsed, expected);
        }
    }
}
