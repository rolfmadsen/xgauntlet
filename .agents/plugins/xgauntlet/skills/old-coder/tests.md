# Good and Bad Tests

## Good Tests

**Integration-style**: Test through real interfaces, not mocks of internal parts.

```rust
// GOOD: Tests observable behavior through public interface
#[test]
fn user_can_checkout_with_valid_cart() {
    let mut cart = Cart::new();
    cart.add_item(Item::new("book", 1500));
    let result = checkout(&cart, &PaymentMethod::Cash);
    assert_eq!(result.status(), Status::Confirmed);
}
```

Characteristics:
- Tests behavior users/callers care about
- Uses public API only
- Survives internal refactors
- Describes WHAT, not HOW
- One logical assertion per test

## Bad Tests

**Implementation-detail tests**: Coupled to internal structure.

Red flags:
- Mocking internal collaborators
- Testing private methods
- Asserting on call counts / call order
- Test breaks during refactor when observable behavior has NOT changed
- Verifying through external bypass (e.g. raw DB table inspect) instead of the module's public interface

**Tautological tests**: Expected value restates the implementation, so the test passes by construction.
- Expected value must come from an independent, known literal or spec, not a re-computation of the code under test.
