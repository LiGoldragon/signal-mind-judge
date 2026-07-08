//! Typed request and reply contract between `mind` and `mind-judge`.
//!
//! The binary wire is rkyv-backed through `signal-frame`. NOTA projection is an
//! edge feature for clients, tests, and tools; it is not the component-to-
//! component transport.

#![forbid(unsafe_code)]

use thiserror::Error;

pub const SIGNAL_SCHEMA_SOURCE: &str = include_str!("../schema/signal.schema");

pub type MindJudgeFrame = signal_frame::ExchangeFrame<MindJudgeRequest, MindJudgeReply>;
pub type MindJudgeFrameBody = signal_frame::ExchangeFrameBody<MindJudgeRequest, MindJudgeReply>;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum Error {
    #[error("mind judge contract value is empty")]
    EmptyValue,
}

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum MindJudgeRequest {
    JudgeKnowledge(KnowledgeJudgmentRequest),
}

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum MindJudgeReply {
    KnowledgeJudged(KnowledgeJudgment),
    RequestRejected(MindJudgeRequestRejection),
}

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct KnowledgeJudgmentRequest {
    submitted_domain: SubmittedKnowledgeDomain,
    submitted_statement: SubmittedKnowledgeStatement,
    relevant_neighbors: RelevantKnowledgeNeighbors,
}

impl KnowledgeJudgmentRequest {
    pub fn new(
        submitted_domain: SubmittedKnowledgeDomain,
        submitted_statement: SubmittedKnowledgeStatement,
        relevant_neighbors: RelevantKnowledgeNeighbors,
    ) -> Self {
        Self {
            submitted_domain,
            submitted_statement,
            relevant_neighbors,
        }
    }

    pub fn submitted_domain(&self) -> &SubmittedKnowledgeDomain {
        &self.submitted_domain
    }

    pub fn submitted_statement(&self) -> &SubmittedKnowledgeStatement {
        &self.submitted_statement
    }

    pub fn relevant_neighbors(&self) -> &RelevantKnowledgeNeighbors {
        &self.relevant_neighbors
    }
}

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct RelevantKnowledgeNeighbors(Vec<RelevantKnowledgeNeighbor>);

impl RelevantKnowledgeNeighbors {
    pub fn new(neighbors: Vec<RelevantKnowledgeNeighbor>) -> Self {
        Self(neighbors)
    }

    pub fn as_slice(&self) -> &[RelevantKnowledgeNeighbor] {
        self.0.as_slice()
    }
}

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct RelevantKnowledgeNeighbor {
    identity: KnowledgeIdentity,
    domain: SubmittedKnowledgeDomain,
    statement: SubmittedKnowledgeStatement,
}

impl RelevantKnowledgeNeighbor {
    pub fn new(
        identity: KnowledgeIdentity,
        domain: SubmittedKnowledgeDomain,
        statement: SubmittedKnowledgeStatement,
    ) -> Self {
        Self {
            identity,
            domain,
            statement,
        }
    }

    pub fn identity(&self) -> &KnowledgeIdentity {
        &self.identity
    }

    pub fn domain(&self) -> &SubmittedKnowledgeDomain {
        &self.domain
    }

    pub fn statement(&self) -> &SubmittedKnowledgeStatement {
        &self.statement
    }
}

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct KnowledgeJudgment {
    verdict: KnowledgeJudgmentVerdict,
    diagnostic_message: OptionalDiagnosticMessage,
}

impl KnowledgeJudgment {
    pub fn new(
        verdict: KnowledgeJudgmentVerdict,
        diagnostic_message: OptionalDiagnosticMessage,
    ) -> Self {
        Self {
            verdict,
            diagnostic_message,
        }
    }

    pub fn verdict(&self) -> KnowledgeJudgmentVerdict {
        self.verdict
    }

    pub fn diagnostic_message(&self) -> Option<&DiagnosticMessage> {
        self.diagnostic_message.as_ref()
    }
}

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum KnowledgeJudgmentVerdict {
    Accept,
    Reject,
}

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct MindJudgeRequestRejection {
    message: DiagnosticMessage,
}

impl MindJudgeRequestRejection {
    pub fn new(message: DiagnosticMessage) -> Self {
        Self { message }
    }

    pub fn message(&self) -> &DiagnosticMessage {
        &self.message
    }
}

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct OptionalDiagnosticMessage(Option<DiagnosticMessage>);

impl OptionalDiagnosticMessage {
    pub fn new(message: Option<DiagnosticMessage>) -> Self {
        Self(message)
    }

    pub fn as_ref(&self) -> Option<&DiagnosticMessage> {
        self.0.as_ref()
    }
}

macro_rules! non_empty_text_type {
    ($name:ident) => {
        #[derive(
            rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq,
        )]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Result<Self, Error> {
                let value = value.into();
                if value.is_empty() {
                    return Err(Error::EmptyValue);
                }
                Ok(Self(value))
            }

            pub fn as_str(&self) -> &str {
                self.0.as_str()
            }
        }
    };
}

non_empty_text_type!(SubmittedKnowledgeDomain);
non_empty_text_type!(SubmittedKnowledgeStatement);
non_empty_text_type!(KnowledgeIdentity);
non_empty_text_type!(DiagnosticMessage);
