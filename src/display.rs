extern crate chrono;

use chrono::prelude::*;
use crate::terror::*;

pub fn display_local_timestamp(utc_date_time: &str) -> Result<String, TError> {
    let parsed_end_time = DateTime::parse_from_rfc3339(utc_date_time)?;
    let converted_date_time = DateTime::<Local>::from(parsed_end_time);

    Ok(converted_date_time.format("%H:%M:%S").to_string())
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_local_timestamp() {
        let timest = FixedOffset::east(0).ymd(1983, 4, 13).and_hms_milli(12, 9, 14, 274).to_rfc3339();
        assert_eq!(display_local_timestamp(&timest).unwrap(), "22:09:14");
    }

    #[test]
    fn test_error_hit() {
        let timest = "arestneasrtn";
        assert_eq!(display_local_timestamp(&timest).is_err(), true);
    }
}
