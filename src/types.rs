/// Common types used in JSON APIs

pub type Decimal = rust_decimal::Decimal;
pub type CountryCode = isocountry::CountryCode;
pub type Date = time::Date;

/// Use the `serde-well-known` feature from `time` to determine format!
/// to convert from/to ISO8601 / RFC3339 add one the following attributes to your struct fields:
/// #[serde(with = "time::serde::rfc3339")]
/// #[serde(with = "time::serde::rfc3339::option")]
/// to convert from/to UNIX Timestamp add one the following attributes to your struct fields:
/// #[serde(with = "time::serde::timestamp")]
/// #[serde(with = "time::serde::timestamp::option")]
pub type DateTime = time::OffsetDateTime;

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use serde_json::json;

    #[test]
    fn test_decimal() {
        let got: Decimal = serde_json::from_value(json!(0.25)).unwrap();
        let expected = Decimal::from_f32_retain(0.25).unwrap();
        assert_eq!(got, expected);
    }


    #[test]
    fn test_countrycode() {
        let got: CountryCode = serde_json::from_value(json!("HU")).unwrap();
        let expected = CountryCode::HUN;
        assert_eq!(got, expected);
    }

    #[test]
    fn test_date() {
        let got: Date = serde_json::from_value(json!("2022-01-05")).unwrap();
        let expected = Date::from_calendar_date(2022, time::Month::January, 5).unwrap();
        assert_eq!(got, expected);
    }

    #[derive(Deserialize, PartialEq, Eq, Debug)]
    struct ContainsDateTime {
        #[serde(with = "time::serde::rfc3339")]
        pub datetime: DateTime,
    }

    #[derive(Deserialize, PartialEq, Eq, Debug)]
    struct ContainsOptionalDateTime {
        #[serde(with = "time::serde::rfc3339::option")]
        pub datetime: Option<DateTime>,
    }

    #[test]
    fn test_datetime_rfc3339() {
        let got: ContainsDateTime = serde_json::from_value(json!({ "datetime": "2022-01-05T06:24:24.253-08:00" })).unwrap();
        let expected_date = Date::from_calendar_date(2022, time::Month::January, 5).unwrap();
        let expected_time = time::Time::from_hms_milli(6, 24, 24, 253).unwrap();
        let expected_primitive = time::PrimitiveDateTime::new(expected_date, expected_time);
        let expected_offset = time::UtcOffset::from_hms(-8, 0, 0).unwrap();
        let expected_odt = expected_primitive.assume_offset(expected_offset);
        let expected = ContainsDateTime { datetime: expected_odt };
        assert_eq!(got, expected);
    }

    #[test]
    fn test_datetime_rfc3339_option() {
        let got: ContainsOptionalDateTime = serde_json::from_value(json!({ "datetime": "2022-01-05T06:24:24.253-08:00" })).unwrap();
        let expected_date = Date::from_calendar_date(2022, time::Month::January, 5).unwrap();
        let expected_time = time::Time::from_hms_milli(6, 24, 24, 253).unwrap();
        let expected_primitive = time::PrimitiveDateTime::new(expected_date, expected_time);
        let expected_offset = time::UtcOffset::from_hms(-8, 0, 0).unwrap();
        let expected_odt = expected_primitive.assume_offset(expected_offset);
        let expected = ContainsOptionalDateTime { datetime: Some(expected_odt) };
        assert_eq!(got, expected);

        let got2: ContainsOptionalDateTime = serde_json::from_value(json!({ "datetime": null })).unwrap();
        assert_eq!(got2, ContainsOptionalDateTime { datetime: None });
    }

    #[derive(Deserialize, PartialEq, Eq, Debug)]
    struct ContainsTimestamp {
        #[serde(with = "time::serde::timestamp")]
        pub datetime: DateTime,
    }

    #[derive(Deserialize, PartialEq, Eq, Debug)]
    struct ContainsOptionalTimestamp {
        #[serde(with = "time::serde::timestamp::option")]
        pub datetime: Option<DateTime>,
    }

    #[test]
    fn test_datetime_timestamp() {
        let got: ContainsTimestamp = serde_json::from_value(json!({ "datetime": 1643053306 })).unwrap();
        let expected_date = Date::from_calendar_date(2022, time::Month::January, 24).unwrap();
        let expected_time = time::Time::from_hms(19, 41, 46).unwrap();
        let expected_primitive = time::PrimitiveDateTime::new(expected_date, expected_time);
        let expected_offset = time::UtcOffset::from_hms(0,0,0).unwrap();
        let expected_odt = expected_primitive.assume_offset(expected_offset);
        let expected = ContainsTimestamp { datetime: expected_odt };
        assert_eq!(got, expected);
    }

    #[test]
    fn test_datetime_timestamp_option() {
        let got: ContainsOptionalTimestamp = serde_json::from_value(json!({ "datetime": 1643053306 })).unwrap();
        let expected_date = Date::from_calendar_date(2022, time::Month::January, 24).unwrap();
        let expected_time = time::Time::from_hms(19, 41, 46).unwrap();
        let expected_primitive = time::PrimitiveDateTime::new(expected_date, expected_time);
        let expected_offset = time::UtcOffset::from_hms(0,0,0).unwrap();
        let expected_odt = expected_primitive.assume_offset(expected_offset);
        let expected = ContainsOptionalTimestamp { datetime: Some(expected_odt) };
        assert_eq!(got, expected);

        let got2: ContainsOptionalTimestamp = serde_json::from_value(json!({ "datetime": null })).unwrap();
        assert_eq!(got2, ContainsOptionalTimestamp { datetime: None });
    }
}
