# signal-mind-judge

Typed Signal contract between `mind` and the `mind-judge` adapter.

The contract owns both request and reply records for Mind judgment calls. The
binary wire is rkyv-backed; NOTA projection is only for clients, tests, and
tools.
