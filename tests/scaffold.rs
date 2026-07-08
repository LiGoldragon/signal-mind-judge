use signal_mind_judge::{
    KnowledgeJudgmentRequest, RelevantKnowledgeNeighbors, SubmittedKnowledgeDomain,
    SubmittedKnowledgeStatement,
};

#[test]
fn knowledge_judgment_request_names_the_core_operation_payload() {
    let request = KnowledgeJudgmentRequest::new(
        SubmittedKnowledgeDomain::new("Technology").unwrap(),
        SubmittedKnowledgeStatement::new("Typed records cross the wire.").unwrap(),
        RelevantKnowledgeNeighbors::new(Vec::new()),
    );

    assert_eq!(request.submitted_domain().as_str(), "Technology");
}
