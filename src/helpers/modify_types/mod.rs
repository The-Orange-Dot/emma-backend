use uuid::{Uuid, Error};

pub fn string_to_uuid(s: String) -> Result<Uuid, Error> {
    Uuid::parse_str(&s)
}