pub trait Delta: Eq {
    fn delta(&self, other: impl Delta) -> Option<()> {
        if self == other {
            return None;
        }
        Some(())
    }
}

struct Record {
    first_name: String,
    email_address: String,
}

impl Delta for Record {}

pub fn main() {
    let r1 = Record {
        first_name: "Susan".into(),
        email_address: "susan@example.com".into(),
    };

    let r2 = Record {
        first_name: "Susan".into(),
        email_address: "susan.b@example.com".into(),
    };

    let changed_fields = r2.delta(r1);
}
