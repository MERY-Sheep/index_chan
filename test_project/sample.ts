// Used function
export function usedFunction() {
    console.log("This is used - updated!");
    helperFunction();
}

// Helper function (used internally)
function helperFunction() {
    console.log("Helper");
}

// Dead code - not used anywhere
// @ts-ignore - index-chan: Test file - may be used in tests
function deadFunction() {
    console.log("This is dead code");
}

// Dead code - exported but not used
export function unusedExport() {
    console.log("Exported but unused");
}

// Class with dead methods
class MyClass {
    usedMethod() {
        console.log("Used");
    }
    
// @ts-ignore - index-chan: Test file - may be used in tests
    deadMethod() {
        console.log("Dead");
    }
}

const instance = new MyClass();
instance.usedMethod();