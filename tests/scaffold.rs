use signal_frame::{
    ExchangeFrameBody, ExchangeIdentifier, ExchangeLane, LaneSequence, NonEmpty, Reply, Request,
    ShortHeader, SubReply,
};
use signal_mind_judge::{
    KnowledgeIdentity, KnowledgeJudgePacket, KnowledgeJudgeResponse, KnowledgeJudgeVerdict,
    KnowledgeRecord, KnowledgeRejectionReason, MindJudgeFrame, MindJudgeFrameCodec, MindJudgeReply,
    MindJudgeRequest, TextBody,
};

fn technology_domain() -> signal_domain::Domain {
    signal_domain::Domain::Technology(signal_domain::Technology::Software(
        signal_domain::Software::Intelligence(signal_domain::IntelligenceLeaf::AgentSystems),
    ))
}

fn exchange_identifier() -> ExchangeIdentifier {
    ExchangeIdentifier::new(
        signal_frame::SessionEpoch::new(7),
        ExchangeLane::Connector,
        LaneSequence::first(),
    )
}

#[test]
fn judge_knowledge_packet_names_the_core_operation_payload() {
    let packet = KnowledgeJudgePacket::new(
        technology_domain(),
        TextBody::new("Typed records cross the wire.").unwrap(),
        Vec::new(),
    );

    assert_eq!(packet.statement.as_str(), "Typed records cross the wire.");
}

#[test]
fn judge_knowledge_request_round_trips_through_binary_frame() {
    let request = MindJudgeRequest::JudgeKnowledge(KnowledgeJudgePacket::new(
        technology_domain(),
        TextBody::new("Mind records durable intent as knowledge.").unwrap(),
        vec![KnowledgeRecord::new(
            KnowledgeIdentity::new("knowledge-1").unwrap(),
            technology_domain(),
            TextBody::new("Mind keeps accepted knowledge typed.").unwrap(),
        )],
    ));
    let frame = MindJudgeFrame::with_short_header(
        ShortHeader::new(1),
        ExchangeFrameBody::Request {
            exchange: exchange_identifier(),
            request: Request::from_payload(request.clone()),
        },
    );

    let encoded = frame.encode_length_prefixed().unwrap();
    let decoded = MindJudgeFrame::decode_length_prefixed(&encoded).unwrap();

    assert_eq!(decoded.body(), frame.body());
}

#[test]
fn frame_codec_echoes_request_exchange_in_reply_frame() {
    let codec = MindJudgeFrameCodec::with_exchange(1024 * 1024, exchange_identifier());
    let request = MindJudgeRequest::JudgeKnowledge(KnowledgeJudgePacket::new(
        technology_domain(),
        TextBody::new("The contract owns frame exchange correlation.").unwrap(),
        Vec::new(),
    ));

    let request_frame = codec.request_frame(request.clone());
    let received = codec.request_from_frame(request_frame).unwrap();
    let reply = MindJudgeReply::KnowledgeJudged(KnowledgeJudgeResponse::new(
        KnowledgeJudgeVerdict::Accept,
        None,
    ));
    let reply_frame = received.reply_frame(reply.clone());

    match reply_frame.into_body() {
        ExchangeFrameBody::Reply { exchange, reply: _ } => {
            assert_eq!(exchange, exchange_identifier())
        }
        _ => panic!("expected reply frame"),
    }
    assert_eq!(received.into_request(), request);
}

#[test]
fn knowledge_judged_reply_round_trips_through_binary_frame() {
    let reply_payload = MindJudgeReply::KnowledgeJudged(KnowledgeJudgeResponse::new(
        KnowledgeJudgeVerdict::Reject(KnowledgeRejectionReason::SemanticDuplicate(
            KnowledgeIdentity::new("knowledge-1").unwrap(),
        )),
        Some(TextBody::new("Existing accepted knowledge already covers it.").unwrap()),
    ));
    let frame = MindJudgeFrame::with_short_header(
        ShortHeader::new(1),
        ExchangeFrameBody::Reply {
            exchange: exchange_identifier(),
            reply: Reply::committed(NonEmpty::single(SubReply::Ok(reply_payload))),
        },
    );

    let encoded = frame.encode_length_prefixed().unwrap();
    let decoded = MindJudgeFrame::decode_length_prefixed(&encoded).unwrap();

    assert_eq!(decoded.body(), frame.body());
}

#[cfg(feature = "nota-text")]
#[test]
fn nota_projection_names_knowledge_response_shape() {
    use nota::NotaEncode;

    let reply = MindJudgeReply::KnowledgeJudged(KnowledgeJudgeResponse::new(
        KnowledgeJudgeVerdict::Accept,
        Some(TextBody::new("Accepted as new knowledge.").unwrap()),
    ));

    let text = reply.to_nota();

    assert!(text.contains("KnowledgeJudged"));
    assert!(text.contains("Accept"));
    assert!(text.contains("Accepted as new knowledge."));
}
