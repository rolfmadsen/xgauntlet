# When to Mock

Mock at **system boundaries** only:
- External APIs (payment providers, third-party webhooks)
- External time / randomness
- Truly external unowned services

Don't mock:
- Your own internal modules/classes
- Internal collaborators
- Anything you control

## Designing for Mockability

At system boundaries, design interfaces that make testing natural:

1. **Accept dependencies, don't create them**: pass trait implementations or clients into functions rather than instantiating them internally.
2. **Ports & Adapters**: define an interface trait at the boundary; provide an in-memory or fake adapter for tests, and the real network adapter for production.
3. **One adapter = hypothetical seam; two adapters = real seam**: don't add a mockable seam unless at least two implementations genuinely exist (typically production + test).
