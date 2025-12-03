// Test file for undo functionality

export function usedFunction() {
    console.log("This is used");
    helperFunction();
}

function helperFunction() {
    console.log("Helper");
}

function deadFunction() {
    console.log("Dead code");
}

export function unusedExport() {
    console.log("Unused export");
}
