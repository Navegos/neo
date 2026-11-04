# #0032 Antigravity eligibility check returns 404 not found Google upstream service endpoint not found

- 2026-11-04T16:13:34Z `issue`: Antigravity eligibility check returns 404 not found Google upstream service endpoint not found [crates/toksi/src/headroom/proxy.rs:672 (serve_headroom_connection)]
- 2026-11-04T16:34:14Z `attempt`: Implemented intelligent sibling upstream fallback between aicode.googleapis.com and cloudcode-pa.googleapis.com on 404, fixed primary discrimination for v1internal vs v1main, and added proxy route logging (worked)
- 2026-11-04T16:34:30Z `fix`: Automatic sibling upstream fallback between aicode.googleapis.com and cloudcode-pa.googleapis.com on 404, eliminating 404 endpoint not found during Antigravity eligibility checks
