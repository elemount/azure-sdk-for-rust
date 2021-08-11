pub mod access_tier;
pub mod requests;
pub mod responses;

use crate::parsing_xml::{cast_must, cast_optional, traverse};
use access_tier::AccessTier;
use azure_core::headers::{
    META_PREFIX, SHARE_NEXT_ALLOWED_QUOTA_DOWNGRADE_TIME, SHARE_PROVISIONED_EGRESS_MBPS,
    SHARE_PROVISIONED_INGRESS_MBPS, SHARE_PROVISIONED_IOPS, SHARE_QUOTA, STORAGE_ACCESS_TIER,
};
use azure_core::incompletevector::IncompleteVector;
use chrono::{DateTime, Utc};
use http::request::Builder;
use http::{header, HeaderMap};
use std::collections::HashMap;
use std::str::FromStr;
use xml::{Element, Xml};

#[derive(Debug, Clone)]
pub struct Share {
    pub name: String,
    pub snapshot: Option<DateTime<Utc>>,
    pub version: Option<String>,
    pub deleted: bool,
    pub last_modified: DateTime<Utc>,
    pub e_tag: String,
    pub quota: u64,
    pub provisioned_iops: Option<u64>,
    pub provisioned_ingress_mbps: Option<u64>,
    pub provisioned_egress_mbps: Option<u64>,
    pub next_allowed_quota_downgrade_time: Option<DateTime<Utc>>,
    pub deleted_time: Option<DateTime<Utc>>,
    pub remaining_retention_days: Option<u64>,
    pub access_tier: AccessTier,
    pub metadata: HashMap<String, String>,
}

impl AsRef<str> for Share {
    fn as_ref(&self) -> &str {
        &self.name
    }
}

impl Share {
    pub fn new(name: &str) -> Share {
        Share {
            name: name.to_owned(),
            snapshot: None,
            version: None,
            deleted: false,
            last_modified: Utc::now(),
            e_tag: "".to_owned(),
            quota: 0,
            provisioned_iops: None,
            provisioned_ingress_mbps: None,
            provisioned_egress_mbps: None,
            next_allowed_quota_downgrade_time: None,
            deleted_time: None,
            remaining_retention_days: None,
            access_tier: AccessTier::TransactionOptimized,
            metadata: HashMap::new(),
        }
    }

    pub(crate) fn from_response<NAME>(
        name: NAME,
        headers: &HeaderMap,
    ) -> Result<Share, crate::Error>
    where
        NAME: Into<String>,
    {
        let last_modified = match headers.get(header::LAST_MODIFIED) {
            Some(last_modified) => last_modified.to_str()?,
            None => {
                static LM: header::HeaderName = header::LAST_MODIFIED;
                return Err(crate::Error::MissingHeaderError(LM.as_str().to_owned()));
            }
        };
        let last_modified = DateTime::parse_from_rfc2822(last_modified)?;
        let last_modified = DateTime::from_utc(last_modified.naive_utc(), Utc);
        let e_tag = match headers.get(header::ETAG) {
            Some(e_tag) => e_tag.to_str()?.to_owned(),
            None => {
                return Err(crate::Error::MissingHeaderError(
                    header::ETAG.as_str().to_owned(),
                ));
            }
        };

        let access_tier = match headers.get(STORAGE_ACCESS_TIER) {
            Some(access_tier) => access_tier.to_str()?,
            None => {
                return Err(crate::Error::MissingHeaderError(
                    STORAGE_ACCESS_TIER.to_owned(),
                ))
            }
        };
        let access_tier = AccessTier::from_str(access_tier)?;

        let quota = match headers.get(SHARE_QUOTA) {
            Some(quota_str) => quota_str.to_str()?.parse::<u64>()?,
            None => {
                return Err(crate::Error::MissingHeaderError(SHARE_QUOTA.to_owned()));
            }
        };

        let provisioned_iops = match headers.get(SHARE_PROVISIONED_IOPS) {
            Some(value) => Some(value.to_str()?.parse::<u64>()?),
            None => None,
        };

        let provisioned_ingress_mbps = match headers.get(SHARE_PROVISIONED_INGRESS_MBPS) {
            Some(value) => Some(value.to_str()?.parse::<u64>()?),
            None => None,
        };

        let provisioned_egress_mbps = match headers.get(SHARE_PROVISIONED_EGRESS_MBPS) {
            Some(value) => Some(value.to_str()?.parse::<u64>()?),
            None => None,
        };

        let next_allowed_quota_downgrade_time =
            match headers.get(SHARE_NEXT_ALLOWED_QUOTA_DOWNGRADE_TIME) {
                Some(value) => Some(DateTime::from_utc(
                    DateTime::parse_from_rfc2822(value.to_str()?)?.naive_utc(),
                    Utc,
                )),
                None => None,
            };

        let mut metadata: HashMap<String, String> = HashMap::new();
        for (key, value) in headers {
            if key.as_str().starts_with(META_PREFIX) {
                metadata.insert(key.as_str().to_owned(), value.to_str()?.to_owned());
            }
        }

        Ok(Share {
            name: name.into(),
            last_modified,
            e_tag,
            access_tier,
            quota,
            provisioned_iops,
            provisioned_ingress_mbps,
            provisioned_egress_mbps,
            next_allowed_quota_downgrade_time,
            metadata,
            snapshot: None,
            version: None,
            deleted: false,
            deleted_time: None,
            remaining_retention_days: None,
        })
    }

    fn parse(elem: &Element) -> Result<Share, crate::Error> {
        let name = cast_must::<String>(elem, &["Name"])?;
        let snapshot = cast_optional::<DateTime<Utc>>(elem, &["Snapshot"])?;
        let version = cast_optional::<String>(elem, &["Version"])?;
        let deleted = match cast_optional::<bool>(elem, &["Deleted"])? {
            Some(deleted_status) => deleted_status,
            None => false,
        };

        let last_modified = cast_must::<DateTime<Utc>>(elem, &["Properties", "Last-Modified"])?;
        let e_tag = cast_must::<String>(elem, &["Properties", "Etag"])?;
        let quota = cast_must::<u64>(elem, &["Properties", "Quota"])?;
        let deleted_time = cast_optional::<DateTime<Utc>>(elem, &["Properties", "DeletedTime"])?;
        let remaining_retention_days =
            cast_optional::<u64>(elem, &["Properties", "RemainingRetentionDays"])?;
        let access_tier = cast_must::<AccessTier>(elem, &["Properties", "AccessTier"])?;

        let metadata = {
            let mut hm = HashMap::new();
            let metadata = traverse(elem, &["Metadata"], true)?;

            for m in metadata {
                for key in &m.children {
                    let elem = match key {
                        Xml::ElementNode(elem) => elem,
                        _ => {
                            return Err(crate::Error::UnexpectedXMLError(String::from(
                                "Metadata should contain an ElementNode",
                            )));
                        }
                    };

                    let key = elem.name.to_owned();

                    if elem.children.is_empty() {
                        return Err(crate::Error::UnexpectedXMLError(String::from(
                            "Metadata node should not be empty",
                        )));
                    }

                    let content = {
                        match elem.children[0] {
                            Xml::CharacterNode(ref content) => content.to_owned(),
                            _ => {
                                return Err(crate::Error::UnexpectedXMLError(String::from(
                                    "Metadata node should contain a CharacterNode with metadata value",
                                )));
                            }
                        }
                    };

                    hm.insert(key, content);
                }
            }

            hm
        };

        Ok(Share {
            name,
            snapshot,
            version,
            deleted,
            last_modified,
            e_tag,
            quota,
            deleted_time,
            remaining_retention_days,
            access_tier,
            metadata,
            provisioned_iops: None,
            provisioned_ingress_mbps: None,
            provisioned_egress_mbps: None,
            next_allowed_quota_downgrade_time: None,
        })
    }
}

pub(crate) fn incomplete_vector_from_share_response(
    body: &str,
) -> Result<IncompleteVector<Share>, crate::Error> {
    let elem: Element = body.parse()?;

    let mut v = Vec::new();

    for share in traverse(&elem, &["Shares", "Share"], true)? {
        v.push(Share::parse(share)?);
    }

    let next_marker = match cast_optional::<String>(&elem, &["NextMarker"])? {
        Some(ref nm) if nm.is_empty() => None,
        Some(nm) => Some(nm.into()),
        None => None,
    };

    Ok(IncompleteVector::new(next_marker, v))
}
