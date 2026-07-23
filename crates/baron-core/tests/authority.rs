use baron_core::authority::{classify_request, RequestAuthority};

#[test]
fn classifies_read_only_requests_without_mutation_authority() {
    for request in [
        "Explain how the memory firewall works",
        "Review this code and report findings only",
        "Diagnose why the installer failed",
        "Show current project status",
        "Lên plan cho thay đổi này nhưng chưa implement",
    ] {
        let decision = classify_request(request);
        assert_eq!(decision.authority, RequestAuthority::ReadOnly, "{request}");
        assert!(!decision.mutation_allowed(), "{request}");
        assert!(!decision.reason.trim().is_empty());
    }
}

#[test]
fn explicit_change_outcome_wins_over_review_or_diagnosis_words() {
    for request in [
        "Review the auth code and apply every valid fix",
        "Diagnose the crash then fix it",
        "Kiểm tra rồi sửa lỗi đăng nhập cho anh",
        "Đánh giá và cập nhật code theo kết quả",
    ] {
        let decision = classify_request(request);
        assert_eq!(decision.authority, RequestAuthority::Change, "{request}");
        assert!(decision.mutation_allowed(), "{request}");
        assert!(!decision.matched_change_terms.is_empty(), "{request}");
    }
}

#[test]
fn ambiguous_requests_default_to_no_mutation() {
    for request in ["Take a look at auth", "Login flow", "Xem cái này", "auth"] {
        let decision = classify_request(request);
        assert_eq!(decision.authority, RequestAuthority::Ambiguous, "{request}");
        assert!(!decision.mutation_allowed(), "{request}");
        assert!(decision.next_action.contains("explicit"));
    }
}

#[test]
fn direct_build_requests_are_change_authorized() {
    for request in [
        "Implement backend login with Gin",
        "Create a mobile app shell",
        "Refactor the parser",
        "Triển khai API Rust",
        "Thêm màn hình đăng nhập",
        "Xóa code legacy đã được xác nhận",
    ] {
        assert_eq!(
            classify_request(request).authority,
            RequestAuthority::Change,
            "{request}"
        );
    }
}
