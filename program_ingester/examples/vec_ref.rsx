use std::io::{BufRead, BufReader, Read};

fn main() {}

struct Thing {
    value: String,
}

fn blah(mut reader: BufReader<impl Read>) -> Result<Vec<&'static Thing>, std::io::Error> {
    let mut features = vec![];
    let mut buf = String::new();
    // while reader.read_line(&mut buf)? > 0 {
    //     //{
    //     let line = buf.trim_end();
    //     let feature = &Thing { value: line.into() };
    //     features.push(feature);
    //     //}
    //     buf.clear();
    // }

    reader
        .lines() //
        .flat_map(|line| match line {
            Ok(s) => Some(s),
            Err(e) => {
                todo!("tracing::error");
                None
            }
        })
        .map(|s| features.push(&Thing { value: s }));
    //.map(|line| line.unwrap()) //

    Ok(features)
}
