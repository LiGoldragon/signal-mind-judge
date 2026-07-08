//! Typed request and reply contract between `mind` and `mind-judge`.
//!
//! The binary wire is rkyv-backed through `signal-frame`. NOTA projection is an
//! edge feature for clients, tests, and tools; it is not the component-to-
//! component transport.

#![forbid(unsafe_code)]

use std::io::{Read, Write};

use signal_frame::{NonEmpty, Reply, RequestPayload, SubReply};
use thiserror::Error;

pub const SIGNAL_SCHEMA_SOURCE: &str = include_str!("../schema/signal.schema");

pub type MindJudgeFrame = signal_frame::ExchangeFrame<MindJudgeRequest, MindJudgeReply>;
pub type MindJudgeFrameBody = signal_frame::ExchangeFrameBody<MindJudgeRequest, MindJudgeReply>;

pub const DEFAULT_MAXIMUM_FRAME_BYTES: usize = 1024 * 1024;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum Error {
    #[error("mind judge contract value is empty")]
    EmptyValue,
}

#[derive(Debug, Error)]
pub enum MindJudgeFrameCodecError {
    #[error("mind judge socket io failed: {0}")]
    Io(#[from] std::io::Error),

    #[error("mind judge frame failed: {0}")]
    Frame(String),

    #[error("unexpected mind judge frame: {0}")]
    UnexpectedFrame(&'static str),

    #[error("mind judge frame has {found} bytes, limit is {limit}")]
    FrameTooLarge { found: usize, limit: usize },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MindJudgeFrameCodec {
    maximum_frame_bytes: usize,
    exchange: signal_frame::ExchangeIdentifier,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MindJudgeReceivedRequest {
    exchange: signal_frame::ExchangeIdentifier,
    request: MindJudgeRequest,
}

impl MindJudgeFrameCodec {
    pub const fn new(maximum_frame_bytes: usize) -> Self {
        Self {
            maximum_frame_bytes,
            exchange: Self::default_exchange(),
        }
    }

    pub const fn with_exchange(
        maximum_frame_bytes: usize,
        exchange: signal_frame::ExchangeIdentifier,
    ) -> Self {
        Self {
            maximum_frame_bytes,
            exchange,
        }
    }

    pub const fn default_exchange() -> signal_frame::ExchangeIdentifier {
        signal_frame::ExchangeIdentifier::new(
            signal_frame::SessionEpoch::new(0),
            signal_frame::ExchangeLane::Connector,
            signal_frame::LaneSequence::first(),
        )
    }

    pub const fn exchange(&self) -> signal_frame::ExchangeIdentifier {
        self.exchange
    }

    pub fn request_frame(&self, request: MindJudgeRequest) -> MindJudgeFrame {
        MindJudgeFrame::new(MindJudgeFrameBody::Request {
            exchange: self.exchange,
            request: request.into_request(),
        })
    }

    pub fn reply_frame(
        &self,
        exchange: signal_frame::ExchangeIdentifier,
        reply: MindJudgeReply,
    ) -> MindJudgeFrame {
        MindJudgeFrame::new(MindJudgeFrameBody::Reply {
            exchange,
            reply: Reply::committed(NonEmpty::single(SubReply::Ok(reply))),
        })
    }

    pub fn decode_frame(
        &self,
        frame_bytes: &[u8],
    ) -> Result<MindJudgeFrame, MindJudgeFrameCodecError> {
        MindJudgeFrame::decode_length_prefixed(frame_bytes)
            .map_err(|error| MindJudgeFrameCodecError::Frame(error.to_string()))
    }

    pub fn encode_frame(
        &self,
        frame: &MindJudgeFrame,
    ) -> Result<Vec<u8>, MindJudgeFrameCodecError> {
        frame
            .encode_length_prefixed()
            .map_err(|error| MindJudgeFrameCodecError::Frame(error.to_string()))
    }

    pub fn read_frame(
        &self,
        reader: &mut impl Read,
    ) -> Result<MindJudgeFrame, MindJudgeFrameCodecError> {
        let bytes = self.read_frame_bytes(reader)?;
        self.decode_frame(bytes.as_slice())
    }

    pub fn write_frame(
        &self,
        writer: &mut impl Write,
        frame: &MindJudgeFrame,
    ) -> Result<(), MindJudgeFrameCodecError> {
        let bytes = self.encode_frame(frame)?;
        writer.write_all(bytes.as_slice())?;
        Ok(())
    }

    pub fn request_from_frame(
        &self,
        frame: MindJudgeFrame,
    ) -> Result<MindJudgeReceivedRequest, MindJudgeFrameCodecError> {
        match frame.into_body() {
            MindJudgeFrameBody::Request { exchange, request } => Ok(MindJudgeReceivedRequest {
                exchange,
                request: request.payloads.into_head(),
            }),
            _ => Err(MindJudgeFrameCodecError::UnexpectedFrame(
                "expected mind judge request",
            )),
        }
    }

    pub fn reply_from_frame(
        &self,
        frame: MindJudgeFrame,
    ) -> Result<MindJudgeReply, MindJudgeFrameCodecError> {
        match frame.into_body() {
            MindJudgeFrameBody::Reply { reply, .. } => match reply {
                Reply::Accepted { per_operation, .. } => match per_operation.into_head() {
                    SubReply::Ok(payload) => Ok(payload),
                    other => Err(MindJudgeFrameCodecError::Frame(format!(
                        "unexpected sub-reply: {other:?}"
                    ))),
                },
                Reply::Rejected { reason } => {
                    Err(MindJudgeFrameCodecError::Frame(reason.to_string()))
                }
            },
            _ => Err(MindJudgeFrameCodecError::UnexpectedFrame(
                "expected mind judge reply",
            )),
        }
    }

    pub fn read_request(
        &self,
        reader: &mut impl Read,
    ) -> Result<MindJudgeReceivedRequest, MindJudgeFrameCodecError> {
        let frame = self.read_frame(reader)?;
        self.request_from_frame(frame)
    }

    pub fn write_request(
        &self,
        writer: &mut impl Write,
        request: MindJudgeRequest,
    ) -> Result<(), MindJudgeFrameCodecError> {
        let frame = self.request_frame(request);
        self.write_frame(writer, &frame)
    }

    pub fn read_reply(
        &self,
        reader: &mut impl Read,
    ) -> Result<MindJudgeReply, MindJudgeFrameCodecError> {
        let frame = self.read_frame(reader)?;
        self.reply_from_frame(frame)
    }

    pub fn write_reply(
        &self,
        writer: &mut impl Write,
        exchange: signal_frame::ExchangeIdentifier,
        reply: MindJudgeReply,
    ) -> Result<(), MindJudgeFrameCodecError> {
        let frame = self.reply_frame(exchange, reply);
        self.write_frame(writer, &frame)
    }

    pub fn frame_payload_length(
        &self,
        length_prefix: [u8; 4],
    ) -> Result<usize, MindJudgeFrameCodecError> {
        let length = u32::from_be_bytes(length_prefix) as usize;
        if length > self.maximum_frame_bytes {
            return Err(MindJudgeFrameCodecError::FrameTooLarge {
                found: length,
                limit: self.maximum_frame_bytes,
            });
        }
        Ok(length)
    }

    pub fn decode_frame_bytes(
        &self,
        length_prefix: [u8; 4],
        payload: Vec<u8>,
    ) -> Result<MindJudgeFrame, MindJudgeFrameCodecError> {
        let length = self.frame_payload_length(length_prefix)?;
        if payload.len() != length {
            return Err(MindJudgeFrameCodecError::Frame(format!(
                "length prefix declared {length} bytes but payload has {} bytes",
                payload.len()
            )));
        }
        let mut bytes = Vec::with_capacity(4 + payload.len());
        bytes.extend_from_slice(&length_prefix);
        bytes.extend_from_slice(payload.as_slice());
        self.decode_frame(bytes.as_slice())
    }

    fn read_frame_bytes(
        &self,
        reader: &mut impl Read,
    ) -> Result<Vec<u8>, MindJudgeFrameCodecError> {
        let mut prefix = [0_u8; 4];
        reader.read_exact(&mut prefix)?;
        let length = self.frame_payload_length(prefix)?;
        let mut bytes = Vec::with_capacity(4 + length);
        bytes.extend_from_slice(&prefix);
        bytes.resize(4 + length, 0);
        reader.read_exact(&mut bytes[4..])?;
        Ok(bytes)
    }
}

impl Default for MindJudgeFrameCodec {
    fn default() -> Self {
        Self::new(DEFAULT_MAXIMUM_FRAME_BYTES)
    }
}

impl MindJudgeReceivedRequest {
    pub fn exchange(&self) -> signal_frame::ExchangeIdentifier {
        self.exchange
    }

    pub fn request(&self) -> &MindJudgeRequest {
        &self.request
    }

    pub fn into_request(self) -> MindJudgeRequest {
        self.request
    }

    pub fn reply_frame(&self, reply: MindJudgeReply) -> MindJudgeFrame {
        MindJudgeFrame::new(MindJudgeFrameBody::Reply {
            exchange: self.exchange,
            reply: Reply::committed(NonEmpty::single(SubReply::Ok(reply))),
        })
    }
}

#[cfg_attr(
    feature = "nota-text",
    derive(nota::NotaDecode, nota::NotaDecodeTraced, nota::NotaEncode)
)]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum MindJudgeRequest {
    JudgeKnowledge(KnowledgeJudgePacket),
}

impl signal_frame::RequestPayload for MindJudgeRequest {}

impl signal_frame::LogVariant for MindJudgeRequest {
    fn log_variant(&self) -> u64 {
        match self {
            Self::JudgeKnowledge(_) => 1,
        }
    }
}

#[cfg_attr(
    feature = "nota-text",
    derive(nota::NotaDecode, nota::NotaDecodeTraced, nota::NotaEncode)
)]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum MindJudgeReply {
    KnowledgeJudged(KnowledgeJudgeResponse),
    RequestRejected(MindJudgeRequestRejection),
}

#[cfg_attr(
    feature = "nota-text",
    derive(nota::NotaDecode, nota::NotaDecodeTraced, nota::NotaEncode)
)]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct KnowledgeJudgePacket {
    pub domain: signal_domain::Domain,
    pub statement: TextBody,
    pub relevant_neighbors: Vec<KnowledgeRecord>,
}

impl KnowledgeJudgePacket {
    pub fn new(
        domain: signal_domain::Domain,
        statement: TextBody,
        relevant_neighbors: Vec<KnowledgeRecord>,
    ) -> Self {
        Self {
            domain,
            statement,
            relevant_neighbors,
        }
    }
}

#[cfg_attr(
    feature = "nota-text",
    derive(nota::NotaDecode, nota::NotaDecodeTraced, nota::NotaEncode)
)]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct KnowledgeRecord {
    pub identity: KnowledgeIdentity,
    pub domain: signal_domain::Domain,
    pub statement: TextBody,
}

impl KnowledgeRecord {
    pub fn new(
        identity: KnowledgeIdentity,
        domain: signal_domain::Domain,
        statement: TextBody,
    ) -> Self {
        Self {
            identity,
            domain,
            statement,
        }
    }
}

#[cfg_attr(
    feature = "nota-text",
    derive(nota::NotaDecode, nota::NotaDecodeTraced, nota::NotaEncode)
)]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct KnowledgeJudgeResponse {
    pub verdict: KnowledgeJudgeVerdict,
    pub diagnostic_message: Option<TextBody>,
}

impl KnowledgeJudgeResponse {
    pub fn new(verdict: KnowledgeJudgeVerdict, diagnostic_message: Option<TextBody>) -> Self {
        Self {
            verdict,
            diagnostic_message,
        }
    }
}

#[cfg_attr(
    feature = "nota-text",
    derive(nota::NotaDecode, nota::NotaDecodeTraced, nota::NotaEncode)
)]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum KnowledgeJudgeVerdict {
    Accept,
    Reject(KnowledgeRejectionReason),
}

#[cfg_attr(
    feature = "nota-text",
    derive(nota::NotaDecode, nota::NotaDecodeTraced, nota::NotaEncode)
)]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum KnowledgeRejectionReason {
    NotKnowledge,
    PrivateOrUnauthorized,
    MeaningUnclear,
    SemanticDuplicate(KnowledgeIdentity),
    ConflictsAcceptedKnowledge(Vec<KnowledgeIdentity>),
    WrongDomain(signal_domain::Domain),
    NeedsMoreSpecificShape,
}

#[cfg_attr(
    feature = "nota-text",
    derive(nota::NotaDecode, nota::NotaDecodeTraced, nota::NotaEncode)
)]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct MindJudgeRequestRejection {
    pub reason: MindJudgeRequestRejectionReason,
    pub message: TextBody,
}

impl MindJudgeRequestRejection {
    pub fn new(reason: MindJudgeRequestRejectionReason, message: TextBody) -> Self {
        Self { reason, message }
    }
}

#[cfg_attr(
    feature = "nota-text",
    derive(nota::NotaDecode, nota::NotaDecodeTraced, nota::NotaEncode)
)]
#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum MindJudgeRequestRejectionReason {
    InvalidRequest,
    ConfigurationUnavailable,
    ProviderUnavailable,
    ProviderRejected,
    ResponseFormatFailure,
}

macro_rules! non_empty_text_type {
    ($name:ident) => {
        #[cfg_attr(
            feature = "nota-text",
            derive(nota::NotaDecode, nota::NotaDecodeTraced, nota::NotaEncode)
        )]
        #[derive(
            rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone, Debug, PartialEq, Eq,
        )]
        pub struct $name(pub String);

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

non_empty_text_type!(KnowledgeIdentity);
non_empty_text_type!(TextBody);
