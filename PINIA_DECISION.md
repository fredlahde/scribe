# Pinia State Management Decision

## Context
The issue #8 suggested considering migrating from the current composable-based store to Pinia for better Vue DevTools integration and patterns.

## Decision
**Decision: Keep the current composable-based approach**

## Reasoning

1. **Current implementation is sufficient**: The `usePendingDelete()` composable is well-structured, maintainable, and follows Vue 3 Composition API best practices.

2. **Minimal state complexity**: The app has only one global state store (pending delete) with simple requirements. Adding Pinia would be over-engineering for this use case.

3. **No significant DevTools benefits**: While Pinia offers excellent DevTools integration, the current implementation is simple enough to debug without special tooling. The reactive state is already visible in Vue DevTools.

4. **Avoiding unnecessary dependencies**: The app currently has a lean dependency footprint. Adding Pinia would increase bundle size for minimal benefit.

5. **Current pattern works well**: The composable pattern with Map-based callback registry for component coordination is clean and testable.

## When to Revisit
Consider migrating to Pinia if:
- The app grows to have 3+ global stores
- Complex state interactions require time-travel debugging
- Team members are more familiar with Pinia patterns
- Additional stores need similar patterns (indicating a need for standardization)

## Recommendation
For future stores, continue using composables with the following pattern:
- Global reactive state with `ref()`
- Composable function returning state and methods
- Clear separation of concerns
- Proper TypeScript typing
