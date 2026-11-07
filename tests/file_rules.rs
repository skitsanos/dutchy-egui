//! Shared metadata acceptance, capacity and size boundaries.
use dutchy_egui::{FileInfo, FileRejectionReason as Reason, FileRules, file_size};
#[test]
fn extensions_and_mime_alternatives_accept_case_parameters_and_compound_suffixes() {
    let rules = FileRules::new()
        .extensions([" .PDF ", "tar.gz"])
        .mime_types(["image/*", "text/plain"]);
    let files = [
        FileInfo::new("REPORT.PDF"),
        FileInfo::new("backup.TAR.GZ"),
        FileInfo::new("scan").mime("IMAGE/PNG; charset=binary"),
        FileInfo::new("note").mime("text/plain"),
        FileInfo::new("executable.exe"),
        FileInfo::new("bad").mime("images/png"),
        FileInfo::new("bad").mime("image/"),
    ];
    let result = rules.validate(&files, 0);
    assert_eq!(result.accepted, [0, 1, 2, 3]);
    assert_eq!(
        result.rejected.iter().map(|r| r.index).collect::<Vec<_>>(),
        [4, 5, 6]
    );
    assert!(
        result
            .rejected
            .iter()
            .all(|r| r.reason == Reason::UnsupportedType)
    );
}
#[test]
fn unknown_sizes_are_explicit_and_limits_include_exact_boundaries() {
    let files = [
        FileInfo::new("unknown"),
        FileInfo::new("empty").size(0),
        FileInfo::new("boundary").size(100),
        FileInfo::new("large").size(101),
    ];
    let result = FileRules::new().max_bytes(100).validate(&files, 0);
    assert_eq!(result.accepted, [1, 2]);
    assert_eq!(result.rejected[0].reason, Reason::UnknownSize);
    assert_eq!(
        result.rejected[1].reason,
        Reason::TooLarge { max_bytes: 100 }
    );
    assert_eq!(
        FileRules::new().max_bytes(0).validate(&files, 0).accepted,
        [1]
    );
    assert_eq!(FileRules::new().validate(&files, 0).accepted, [0, 1, 2, 3]);
}
#[test]
fn rejected_candidates_do_not_consume_capacity_and_retained_count_cannot_overflow() {
    let files = [
        FileInfo::new("wrong.exe").size(1),
        FileInfo::new("ok.pdf").size(1),
        FileInfo::new("next.pdf").size(1),
        FileInfo::new(" "),
    ];
    let rules = FileRules::new().extensions(["pdf"]).max_files(2);
    let result = rules.validate(&files, 1);
    assert_eq!(result.accepted, [1]);
    assert_eq!(result.rejected[1].reason, Reason::TooMany { max_files: 2 });
    assert_eq!(result.rejected[2].reason, Reason::MissingName);
    assert!(rules.validate(&files, usize::MAX).accepted.is_empty());
    assert!(
        FileRules::new()
            .max_files(0)
            .validate(&files, 0)
            .accepted
            .is_empty()
    );
}
#[test]
fn metadata_extraction_does_no_io_and_unknown_sizes_are_not_zero() {
    let info = FileInfo::from_path(std::path::Path::new("/does-not-exist/brief.pdf"));
    assert_eq!(info.name, "brief.pdf");
    assert_eq!(info.bytes, None);
    for (bytes, expected) in [
        (None, "Size unknown"),
        (Some(0), "0 B"),
        (Some(999), "999 B"),
        (Some(1_000), "1.0 KB"),
        (Some(2_400_000), "2.4 MB"),
        (Some(999_949), "999.9 KB"),
        (Some(999_950), "1.0 MB"),
        (Some(999_999_950), "1.0 GB"),
        (Some(u64::MAX), "18.4 EB"),
    ] {
        assert_eq!(file_size(bytes), expected);
    }
}
