use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "LocationInfo")]
pub struct LocationInfo {
    #[serde(rename = "Name")]
    pub name: Option<String>,
    #[serde(rename = "Type")]
    pub location_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "BucketInfo")]
pub struct BucketInfo {
    #[serde(rename = "DataRedundancy")]
    pub data_redundancy: Option<String>,
    #[serde(rename = "Type")]
    pub bucket_type: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn location_info_serializes_correctly() {
        let data = LocationInfo {
            name: Some("hello".into()),
            location_type: Some("Directory".into()),
        };
        let res = quick_xml::se::to_string(&data).unwrap();

        assert_eq!(
            res,
            "<LocationInfo><Name>hello</Name><Type>Directory</Type></LocationInfo>"
        )
    }

    #[test]
    fn bucket_info_serializes_correctly() {
        let data = BucketInfo {
            data_redundancy: Some("hello".into()),
            bucket_type: Some("Directory".into()),
        };
        let res = quick_xml::se::to_string(&data).unwrap();

        assert_eq!(
            res,
            "<BucketInfo><DataRedundancy>hello</DataRedundancy><Type>Directory</Type></BucketInfo>"
        )
    }
}
