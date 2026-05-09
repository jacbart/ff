use super::*;

#[test]
fn test_get_build_info_basic() {
    let info = get_build_info();
    assert!(info.starts_with("ff v"));
    assert!(info.contains("built:"));
}

#[test]
fn test_get_build_info_format_consistency() {
    let info1 = get_build_info();
    let info2 = get_build_info();
    assert_eq!(info1, info2);
}

#[test]
fn test_timestamp_to_date() {
    let date = timestamp_to_date(1640995200); // 2022-01-01
    assert_eq!(date, "2022-01-01");

    let date2 = timestamp_to_date(1704067200); // 2024-01-01
    assert_eq!(date2, "2024-01-01");
}

#[test]
fn test_is_leap_year() {
    assert!(is_leap_year(2020));
    assert!(is_leap_year(2024));
    assert!(!is_leap_year(2021));
    assert!(!is_leap_year(2023));
    assert!(is_leap_year(2000)); // Century leap year
    assert!(!is_leap_year(2100)); // Century non-leap year
}
