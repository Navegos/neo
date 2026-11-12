# #0025 HTTP request parser does not decode Transfer-Encoding chunked, corrupting JSON bodies with chunk headers and failing requests with HTTP 400

- 2026-11-02T16:30:47Z `issue`: HTTP request parser does not decode Transfer-Encoding chunked, corrupting JSON bodies with chunk headers and failing requests with HTTP 400 [crates/toksi/src/headroom/byok_gateway.rs:950 (read_http_request)]
- 2026-11-02T16:33:25Z `attempt`: Implement HTTP chunked transfer decoding in read_http_request and read_chunked_body (worked)
- 2026-11-02T16:33:47Z `fix`: Add Transfer-Encoding chunked decoding loop to read_chunked_body, stripping chunk size headers (e.g. 10103\r\n) and reassembling valid JSON body with Content-Length
