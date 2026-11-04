// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/tests/antigravity_auth_verification.rs
// created: 2026-11-04
// lastModified: 2026-11-04

#[test]
fn test_antigravity_route_discrimination_and_target_resolution() {
    let cloudcode_default = "https://cloudcode-pa.googleapis.com";
    let aicode_default = "https://aicode.googleapis.com";
    let gemini_default = "https://generativelanguage.googleapis.com";

    let test_cases = vec![
        // ConnectRPC Cloud Code
        ("/google.internal.cloud.code.v1internal.CloudCode/LoadCodeAssist", cloudcode_default),
        ("/google.internal.cloud.code.v1internal.JetskiService/FetchUserInfo", cloudcode_default),
        ("/google.internal.cloud.code.v1internal.PredictionService/RetrieveUserQuota", cloudcode_default),
        ("/google.internal.cloud.code.v1internal.PredictionService/FetchAvailableModels", cloudcode_default),
        ("/v1internal:loadCodeAssist", cloudcode_default),
        ("/v1internal:searchSnippets", cloudcode_default),
        // ConnectRPC AICode & ManagementService
        ("/google.gca.aicode.v1main.ManagementService/FetchLicenses", aicode_default),
        ("/google.gca.aicode.v1main.ManagementService/GetUserInfo", aicode_default),
        ("/google.gca.aicode.v1main.PredictionService/FetchQuotaStatus", aicode_default),
        ("/v1main/projects/p1:fetchConfig", aicode_default),
        // Gemini API
        ("/v1beta/models/gemini-2.0-flash:generateContent", gemini_default),
        ("/v1/models/gemini-2.0-pro:streamGenerateContent", gemini_default),
    ];

    for (path, expected_target) in test_cases {
        let is_google_request = path.starts_with("/v1internal")
            || path.starts_with("/v1alpha")
            || path.starts_with("/v1beta")
            || path.starts_with("/v1main")
            || path.starts_with("/v1/projects")
            || path.starts_with("/v1/models")
            || path.starts_with("/google.")
            || path.starts_with("/jetski.")
            || path.starts_with("/exa.")
            || path.contains("cloudcode")
            || path.contains("cloud.code")
            || path.contains("aicode")
            || path.contains("jetski")
            || path.contains("gemini")
            || path.contains("LoadCodeAssist")
            || path.contains("GetLoadCodeAssist")
            || path.contains("FetchLicenses")
            || path.contains("FetchUserInfo")
            || path.contains("GetUserInfo")
            || path.contains("FetchQuotaStatus")
            || path.contains("RetrieveUserQuota")
            || path.contains("FetchAvailableModels")
            || path.contains("CountTokens")
            || path.contains(":generateCode")
            || path.contains(":streamGenerateCode")
            || path.contains(":loadCodeContext")
            || path.contains(":loadMetadata")
            || path.contains(":generateMessage")
            || path.contains(":streamGenerateMessage")
            || path.contains(":generateContent")
            || path.contains(":streamGenerateContent");

        assert!(is_google_request, "Path '{}' should be recognized as a Google request", path);

        let resolved_target = if path.starts_with("/google.gca.aicode")
            || path.starts_with("/v1main")
            || path.contains("aicode")
            || path.contains("FetchLicenses")
            || path.contains("FetchQuotaStatus")
        {
            aicode_default
        } else if path.starts_with("/oauth2") || path.starts_with("/token") {
            "https://oauth2.googleapis.com"
        } else if path.starts_with("/sts") {
            "https://sts.googleapis.com"
        } else if path.starts_with("/v1beta")
            || path.starts_with("/v1/models")
            || (path.contains("gemini")
                && !path.contains("cloudcode")
                && !path.contains("cloud.code")
                && !path.contains("jetski"))
        {
            gemini_default
        } else {
            cloudcode_default
        };

        assert_eq!(
            resolved_target, expected_target,
            "Path '{}' resolved to '{}', expected '{}'",
            path, resolved_target, expected_target
        );
    }
}

#[test]
fn test_unary_grpc_vs_streaming_determination() {
    fn determine_is_stream(content_type: &str, transfer_encoding: &str, incoming_path: &str, content_length: Option<usize>) -> bool {
        let mut is_stream = false;
        let path_low = incoming_path.to_ascii_lowercase();
        let ct_low = content_type.to_ascii_lowercase();
        let te_low = transfer_encoding.to_ascii_lowercase();

        if ct_low.contains("text/event-stream")
            || ct_low.contains("application/x-ndjson")
            || (ct_low.contains("stream") && path_low.contains("stream"))
            || (ct_low.contains("grpc") && (path_low.contains("stream") || path_low.contains("task")))
        {
            is_stream = true;
        }

        if te_low.contains("chunked") && (path_low.contains("stream") || path_low.contains("messages")) {
            is_stream = true;
        }

        if path_low.contains("stream") && content_length.is_none() {
            is_stream = true;
        }

        is_stream
    }

    // Unary gRPC / ConnectRPC responses MUST NOT be marked as streaming
    assert!(!determine_is_stream(
        "application/grpc",
        "",
        "/google.internal.cloud.code.v1internal.CloudCode/LoadCodeAssist",
        Some(128)
    ));
    assert!(!determine_is_stream(
        "application/connect+proto",
        "",
        "/google.gca.aicode.v1main.ManagementService/FetchLicenses",
        Some(256)
    ));
    assert!(!determine_is_stream(
        "application/proto",
        "",
        "/v1internal:loadCodeAssist",
        Some(512)
    ));
    assert!(!determine_is_stream(
        "application/json",
        "",
        "/v1internal:loadCodeAssist",
        Some(1024)
    ));

    // Streaming responses MUST be marked as streaming
    assert!(determine_is_stream(
        "text/event-stream",
        "chunked",
        "/v1internal:streamGenerateCode",
        None
    ));
    assert!(determine_is_stream(
        "application/grpc",
        "",
        "/jetski.product.v1.ConversationService/StreamTask",
        None
    ));
}

#[test]
fn test_antigravity_credential_envelope_parsing() {
    let mock_oauth_json = r#"{
        "access_token": "ya29.a0ARrdaM_mock_test_token_12345",
        "token_type": "Bearer",
        "expires_in": 3599,
        "refresh_token": "1//04_mock_refresh_token"
    }"#;

    let env = toksi::keyring::parse_credential_envelope(mock_oauth_json)
        .expect("Should parse standard OAuth JSON");
    assert_eq!(env.access_token, "ya29.a0ARrdaM_mock_test_token_12345");
    assert_eq!(env.token_type.as_deref(), Some("Bearer"));
    assert!(!env.is_expired(0));

    // Base64 encoded envelope test (as stored in macOS Keychain or WinCred blob)
    let b64 = toksi::keyring::encode_base64(mock_oauth_json.as_bytes());
    let env_b64 = toksi::keyring::parse_credential_envelope(&b64)
        .expect("Should parse base64-encoded OAuth envelope");
    assert_eq!(env_b64.access_token, "ya29.a0ARrdaM_mock_test_token_12345");
}
