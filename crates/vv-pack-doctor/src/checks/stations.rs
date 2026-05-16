//! Station-tag completeness check.
//!
//! Every recipe declares the station it requires via `#station.<name>`.
//! For the gameplay loop to work each declared station must:
//!   - be carried by exactly one object (the actual station block)
//!   - that object must have a `station: (...)` section so the runtime knows
//!     how to open it.

use std::collections::BTreeMap;

use crate::index::{normalize_tag_key, PackIndex};
use crate::report::{Diagnostic, Report};

const CHECK: &str = "stations";

pub fn run(index: &PackIndex<'_>, report: &mut Report) {
    let mut owners: BTreeMap<String, Vec<&crate::scan::ParsedObject>> = BTreeMap::new();
    for obj in &index.scan.objects {
        if let Some(station) = &obj.def.station {
            for tag in &station.station_tags {
                if let Some(rest) = normalize_tag_key(tag)
                    .and_then(|t| t.strip_prefix("station/").map(str::to_string))
                {
                    owners.entry(rest.to_string()).or_default().push(obj);
                }
            }
        }
        for tag in &obj.def.tags {
            if let Some(rest) =
                normalize_tag_key(tag).and_then(|t| t.strip_prefix("station/").map(str::to_string))
            {
                owners.entry(rest.to_string()).or_default().push(obj);
            }
        }
    }

    for (station, objs) in &owners {
        if objs.len() > 1 {
            let ids: Vec<String> = objs.iter().map(|o| o.id.clone()).collect();
            report.error(
                Diagnostic::new(
                    CHECK,
                    format!(
                        "station '{}' is owned by {} objects: {}",
                        station,
                        objs.len(),
                        ids.join(", ")
                    ),
                )
                .with_suggestion("exactly one block must own a given station tag".to_string()),
            );
        }
        for obj in objs {
            if obj.def.station.is_none() {
                report.error(
                    Diagnostic::new(
                        CHECK,
                        format!(
                            "object carries '#core:tag/station/{}' but has no `station: (...)` section",
                            station
                        ),
                    )
                    .with_path(obj.rel_path.clone())
                    .with_id(obj.id.clone())
                    .with_field("station")
                    .with_suggestion(format!(
                        "add a `station: (type: ...)` section or remove the station tag '{}'",
                        station
                    )),
                );
            }
        }
    }
}
