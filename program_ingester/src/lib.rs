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
            2023-01-01T00:00:00.000Z 2023-12-31T00:00:00.000Z program1 InProgress TeamA null->ProductivitySuite
            2023-01-01T00:00:00.000Z 2023-06-30T00:00:00.000Z program1 Complete TeamB ProductivitySuite->Email
            2023-01-01T00:00:00.000Z 2023-04-30T00:00:00.000Z program1 Complete TeamB Email->EmailSearch
            2023-05-01T00:00:00.000Z 2023-06-30T00:00:00.000Z program1 Complete TeamB Email->EmailFilters
        "}
        .trim();

        let expected = Program {
            id: "program1".into(),
            root: Feature {
                id: "ProductivitySuite".into(),
                progress_status: "InProgress".into(),
                assigned_team: "TeamA".into(),
                start_date: DateTime::parse_from_rfc3339("2023-01-01T00:00:00.000Z")
                    .expect("test dates should be checked"),
                end_date: DateTime::parse_from_rfc3339("2023-12-31T00:00:00.000Z")
                    .expect("test dates should be checked"),
                subfeatures: vec![Feature {
                    id: "Email".into(),
                    progress_status: "Complete".into(),
                    assigned_team: "TeamB".into(),
                    start_date: DateTime::parse_from_rfc3339("2023-01-01T00:00:00.000Z")
                        .expect("test dates should be checked"),
                    end_date: DateTime::parse_from_rfc3339("2023-06-30T00:00:00.000Z")
                        .expect("test dates should be checked"),
                    subfeatures: vec![
                        Feature {
                            id: "EmailSearch".into(),
                            progress_status: "Complete".into(),
                            assigned_team: "TeamB".into(),
                            start_date: DateTime::parse_from_rfc3339("2023-01-01T00:00:00.000Z")
                                .expect("test dates should be checked"),
                            end_date: DateTime::parse_from_rfc3339("2023-04-30T00:00:00.000Z")
                                .expect("test dates should be checked"),
                            subfeatures: vec![],
                        },
                        Feature {
                            id: "EmailFilters".into(),
                            progress_status: "Complete".into(),
                            assigned_team: "TeamB".into(),
                            start_date: DateTime::parse_from_rfc3339("2023-05-01T00:00:00.000Z")
                                .expect("test dates should be checked"),
                            end_date: DateTime::parse_from_rfc3339("2023-06-30T00:00:00.000Z")
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
