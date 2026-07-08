# signal-mind-judge — architecture

`signal-mind-judge` is the Mind-specific contract between `mind` and the
`mind-judge` text/model edge adapter.

It owns both sides of the exchange: Mind sends typed judge requests, and the
adapter returns typed judge replies. The initial vertical slice names one core
operation, `JudgeKnowledge`, for accepted-knowledge admission judgment.

## Boundary

Owned here:

- `MindJudgeRequest` and `MindJudgeReply`;
- typed request packet records for Mind knowledge judgment;
- typed verdict and rejection records;
- rkyv-compatible wire records;
- NOTA projection shape for clients, tests, and tools.

Not owned here:

- provider calls and retries, which belong in `judge`;
- prompt prose, which belongs in `mind-judge-config`;
- adapter process lifecycle, which belongs in `mind-judge`;
- Mind storage or admission logic, which belongs in `mind`.
