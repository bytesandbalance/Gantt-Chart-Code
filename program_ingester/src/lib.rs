//! A crate for ingesting Program Features and emitting Hierarchical structures of the programs.
//!
//! # Example
//!
//! ```rust
//! use program_ingester::input::Ingester;
//! use program_ingester::output::ProgramGraph;
//!
//! fn main() -> Result<(), Box<dyn Error>> {
//!
//!   // Get input (eg; file, stdin, socket)
//!   let reader = BufReader::new(input.as_bytes());
//!
//!   // Import features from a BufReader
//!   let ingester = Ingester::try_from(reader)?;
//!
//!   // Produce a graph of programs from those features
//!   let graph = ProgramGraph::from(ingester.features);
//!
//!   // Serialise the output as JSON
//!   let json = serde_json::to_string(&graph)?;
//!
//!   Ok(())
//! }
//! ```

/// docuemtning
pub mod errors;
pub mod input;
pub mod output;

#[cfg(test)]
mod test {
    use chrono::DateTime;
    use indoc::indoc;
    use std::io::BufReader;

    use crate::{
        input::Ingester,
        output::{Feature, Program, ProgramGraph},
    };

    #[test]
    fn test_parsing_multiple_programs() {
        let input = indoc! {"
            2023-01-01T00:00:00.000Z 2023-12-31T00:00:00.000Z program_1 In_Progress Team_A null->Productivity_Suite
            2023-01-01T00:00:00.000Z 2023-06-30T00:00:00.000Z program_1 Complete Team_B Productivity_Suite->Email
            2023-01-01T00:00:00.000Z 2023-04-30T00:00:00.000Z program_1 Complete Team_B Email->Email_Search
            2023-05-01T00:00:00.000Z 2023-06-30T00:00:00.000Z program_1 Complete Team_B Email->Email_Filters
        "}
        .trim();

        let expected = Program {
            id: "program_1".into(),
            root: Feature {
                id: "Productivity_Suite".into(),
                progress_status: "In_Progress".into(),
                assigned_team: "Team_A".into(),
                start: DateTime::parse_from_rfc3339("2023-01-01T00:00:00.000Z")
                    .expect("test dates should be checked"),
                end: DateTime::parse_from_rfc3339("2023-12-31T00:00:00.000Z")
                    .expect("test dates should be checked"),
                subfeatures: vec![Feature {
                    id: "Email".into(),
                    progress_status: "Complete".into(),
                    assigned_team: "Team_B".into(),
                    start: DateTime::parse_from_rfc3339("2023-01-01T00:00:00.000Z")
                        .expect("test dates should be checked"),
                    end: DateTime::parse_from_rfc3339("2023-06-30T00:00:00.000Z")
                        .expect("test dates should be checked"),
                    subfeatures: vec![
                        Feature {
                            id: "Email_Search".into(),
                            progress_status: "Complete".into(),
                            assigned_team: "Team_B".into(),
                            start: DateTime::parse_from_rfc3339("2023-01-01T00:00:00.000Z")
                                .expect("test dates should be checked"),
                            end: DateTime::parse_from_rfc3339("2023-04-30T00:00:00.000Z")
                                .expect("test dates should be checked"),
                            subfeatures: vec![],
                        },
                        Feature {
                            id: "Email_Filter".into(),
                            progress_status: "Complete".into(),
                            assigned_team: "Team_B".into(),
                            start: DateTime::parse_from_rfc3339("2023-05-01T00:00:00.000Z")
                                .expect("test dates should be checked"),
                            end: DateTime::parse_from_rfc3339("2023-06-30T00:00:00.000Z")
                                .expect("test dates should be checked"),
                            subfeatures: vec![],
                        },
                    ],
                }],
            },
        };

        let reader = BufReader::new(input.as_bytes());
        let ingester = Ingester::try_from(reader);

        assert!(ingester.is_ok());
        if let Ok(ingester) = ingester {
            let programgraph = ProgramGraph::from(ingester.features);

            assert_eq!(programgraph.programs.len(), 1);
            assert_eq!(programgraph.programs.get(0), Some(&expected));
        }
    }
}
