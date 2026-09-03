use moto_workshop_lib::domain::{
    invoice::{InvoiceIssue, InvoiceIssueError, InvoiceIssueInput},
    service_visit::ServiceVisitStatus,
};

#[test]
fn invoice_issue_accepts_completed_work_and_calculates_exact_integer_total() {
    // # Arrange
    let input = InvoiceIssueInput {
        invoice_id: 7,
        service_visit_id: 11,
        service_visit_status: ServiceVisitStatus::ReadyForPickup,
        completed_at: Some(1_500),
        issued_at: 1_600,
        labor_charge_fils: 12_500,
        active_part_line_totals_fils: vec![4_500, 7_333],
    };

    // # Act
    let issue = InvoiceIssue::new(input).unwrap();

    // # Assert
    assert_eq!(issue.invoice_number(), "INV-000007");
    assert_eq!(issue.parts_total_fils(), 11_833);
    assert_eq!(issue.total_fils(), 24_333);
    assert_eq!(issue.issued_at(), 1_600);
}

#[test]
fn invoice_issue_rejects_unfinished_cancelled_invalid_and_overflowing_input() {
    // # Arrange
    let valid = InvoiceIssueInput {
        invoice_id: 1,
        service_visit_id: 2,
        service_visit_status: ServiceVisitStatus::Closed,
        completed_at: Some(1_500),
        issued_at: 1_600,
        labor_charge_fils: 1,
        active_part_line_totals_fils: vec![2],
    };

    // # Act / # Assert
    for status in [ServiceVisitStatus::Open, ServiceVisitStatus::Cancelled] {
        assert_eq!(
            InvoiceIssue::new(InvoiceIssueInput {
                service_visit_status: status,
                ..valid.clone()
            }),
            Err(InvoiceIssueError::ServiceVisitNotInvoiceable)
        );
    }
    assert_eq!(
        InvoiceIssue::new(InvoiceIssueInput {
            issued_at: 1_499,
            ..valid.clone()
        }),
        Err(InvoiceIssueError::InvalidTimestamp)
    );
    assert_eq!(
        InvoiceIssue::new(InvoiceIssueInput {
            labor_charge_fils: i64::MAX,
            active_part_line_totals_fils: vec![1],
            ..valid
        }),
        Err(InvoiceIssueError::TotalOverflow)
    );
}
