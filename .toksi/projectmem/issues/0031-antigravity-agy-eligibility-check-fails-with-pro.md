# #0031 Antigravity agy eligibility check fails with proto syntax error line 1:1 invalid value in and out proxy framing

- 2026-11-04T15:37:18Z `issue`: Antigravity agy eligibility check fails with proto syntax error line 1:1 invalid value in and out proxy framing [crates/toksi/src/headroom/proxy.rs:625 (serve_headroom_connection)]
- 2026-11-04T16:01:04Z `attempt`: Added is_proto_or_connect_request, format_connect_error, translated upstream HTML errors into ConnectRPC JSON error format, and routed all cloudcode endpoints to aicode_url (worked)
- 2026-11-04T16:01:21Z `fix`: Preserve binary Protocol Buffer framing in and out, translate upstream 502/HTML errors to structured ConnectRPC error JSON (eliminating proto syntax error), and route all Cloud Code/Jetski RPCs to aicode.googleapis.com
