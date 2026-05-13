//! Surface every parse failure recorded during the scan as a diagnostic.
//!
//! No file is allowed to fail silently. The scanner already attempted both the
//! typed form (`SomeType(...)`) and the bare RON tuple form, so anything that
//! ends up here is genuinely broken.

use crate::report::Report;
use crate::scan::PackScan;

pub fn run(scan: &PackScan, report: &mut Report) {
    for err in &scan.parse_errors {
        report.add_parse_error(err);
    }
}
