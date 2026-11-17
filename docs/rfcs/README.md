# NOVA RFCs (Request for Comments)

This directory contains RFCs for the NOVA programming language.

## RFC Process

1. **Draft**: Initial proposal written as a markdown file
2. **Discussion**: Opened as a GitHub Discussion for community feedback
3. **Review**: Core team reviews and may request changes
4. **Decision**: Accepted, rejected, or deferred
5. **Implementation**: If accepted, implemented and tracked

## RFC Template

```markdown
# RFC: [Title]

- **Status**: [Draft | Accepted | Rejected | Deferred | Implemented]
- **Author(s)**: [Name(s)]
- **Start Date**: YYYY-MM-DD
- **RFC PR**: [Link to PR]

## Summary

One paragraph summary of the proposal.

## Motivation

Why is this change needed? What problems does it solve?

## Detailed Design

Detailed explanation of the design, including:
- Syntax changes
- Semantics
- Type system implications
- Standard library changes
- Tooling implications

## Drawbacks

Why should we NOT do this?

## Alternatives

What other designs were considered?

## Prior Art

How do other languages handle this?

## Unresolved Questions

Open questions that need resolution.

## Future Extensions

Potential future work building on this RFC.
```

## RFC Index

| RFC | Title | Status | Date |
|-----|-------|--------|------|
| 0000 | RFC Process | Active | 2026-01-01 |
| 0001 | Ownership and Borrowing | Accepted | 2026-01-15 |
| 0002 | Async/Await | Accepted | 2026-01-20 |
| 0003 | Algebraic Effects | Draft | 2026-02-01 |
| 0004 | Dependent Types | Draft | 2026-02-15 |
| 0005 | SIMD Intrinsics | Accepted | 2026-03-01 |
| 0006 | Actor Model | Accepted | 2026-03-15 |
| 0007 | Package Manager | Implemented | 2026-04-01 |
| 0008 | Module System | Accepted | 2026-04-15 |
| 0009 | Macro System | Draft | 2026-05-01 |
| 0010 | FFI Improvements | Draft | 2026-05-15 |

## Contributing

To propose a new RFC:

1. Fork the repository
2. Create a new file `rfcs/NNNN-title.md` using the template
3. Open a PR with the draft
4. Engage in discussion

See [RFC 0000](0000-rfc-process.md) for the full process.