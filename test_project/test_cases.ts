// Test cases for dead code detection accuracy

// Case 1: Definitely safe to delete - old deprecated function
function oldDeprecatedFunction() {
    // This was replaced 2 years ago
    return "old implementation";
}

// Case 2: Keep for future - WIP feature
function futureFeature() {
    // TODO: Implement this next week
    // Part of issue #123
    return "coming soon";
}

// Case 3: Actually used - should NOT be detected
export function activeFunction() {
    return "I'm being used!";
}

// Case 4: Experimental - needs review
function experimentalAI() {
    // Experimental feature for AI integration
    // Under discussion in PR #456
    return "experimental";
}

// Case 5: Helper used internally
function internalHelper() {
    return "helper";
}

export function publicAPI() {
    return internalHelper();
}

// Case 6: Dead code - simple unused
function simpleUnused() {
    return 42;
}
