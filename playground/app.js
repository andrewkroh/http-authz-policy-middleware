// Copyright (c) 2025 Andrew Kroh — MIT License

import init, { playground_compile, playground_eval } from "./pkg/traefik_authz_wasm.js";

const $ = (sel) => document.querySelector(sel);
const evaluateBtn = $("#evaluate-btn");
const expressionEl = $("#expression");
const compileStatus = $("#compile-status");
const resultPanel = $("#result-panel");
const headersList = $("#headers-list");
const addHeaderBtn = $("#add-header");

let wasmReady = false;
let compileTimer = null;

// Initialize WASM module
async function start() {
    try {
        await init();
        wasmReady = true;
        evaluateBtn.disabled = false;
        liveCompile();
    } catch (e) {
        compileStatus.textContent = "Failed to load WASM module: " + e.message;
        compileStatus.className = "compile-status invalid";
    }
}

// Live compile validation (debounced)
function liveCompile() {
    if (!wasmReady) return;
    const expr = expressionEl.value.trim();
    if (!expr) {
        compileStatus.textContent = "";
        compileStatus.className = "compile-status";
        return;
    }
    const result = JSON.parse(playground_compile(expr));
    if (result.ok) {
        compileStatus.textContent = "Expression is valid";
        compileStatus.className = "compile-status valid";
    } else {
        compileStatus.textContent = result.error;
        compileStatus.className = "compile-status invalid";
    }
}

expressionEl.addEventListener("input", () => {
    clearTimeout(compileTimer);
    compileTimer = setTimeout(liveCompile, 300);
});

// Evaluate
function evaluate() {
    if (!wasmReady) return;
    const expression = expressionEl.value.trim();
    if (!expression) return;

    const request = {
        method: $("#method").value,
        path: $("#path").value,
        host: $("#host").value,
        headers: collectHeaders(),
    };

    const raw = playground_eval(JSON.stringify({ expression, request }));
    const result = JSON.parse(raw);

    resultPanel.classList.remove("hidden", "allow", "deny", "error");
    if (result.error) {
        resultPanel.textContent = "ERROR: " + result.error;
        resultPanel.classList.add("error");
    } else if (result.result === true) {
        resultPanel.textContent = "ALLOW (true)";
        resultPanel.classList.add("allow");
    } else {
        resultPanel.textContent = "DENY (false)";
        resultPanel.classList.add("deny");
    }
}

evaluateBtn.addEventListener("click", evaluate);

document.addEventListener("keydown", (e) => {
    if ((e.ctrlKey || e.metaKey) && e.key === "Enter") {
        e.preventDefault();
        evaluate();
    }
});

// Headers management
function collectHeaders() {
    const headers = {};
    for (const row of headersList.querySelectorAll(".header-row")) {
        const key = row.querySelector(".header-key").value.trim();
        const value = row.querySelector(".header-value").value;
        if (key) headers[key] = value;
    }
    return headers;
}

function addHeaderRow(key, value) {
    const row = document.createElement("div");
    row.className = "header-row";
    row.innerHTML =
        '<input type="text" class="header-key" placeholder="Header name" value="' + escapeAttr(key) + '">' +
        '<input type="text" class="header-value" placeholder="Value" value="' + escapeAttr(value) + '">' +
        '<button type="button" class="btn-remove" title="Remove header">&times;</button>';
    row.querySelector(".btn-remove").addEventListener("click", () => row.remove());
    headersList.appendChild(row);
}

function escapeAttr(s) {
    return s.replace(/&/g, "&amp;").replace(/"/g, "&quot;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}

addHeaderBtn.addEventListener("click", () => addHeaderRow("", ""));

// Attach remove handler to initial header row
headersList.querySelector(".btn-remove").addEventListener("click", function () {
    this.closest(".header-row").remove();
});

// Footer setup
$("#copyright-year").textContent = new Date().getFullYear();
const commitLink = $("#commit-link");
const sha = commitLink.textContent;
if (sha && sha !== "COMMIT_SHA_SHORT") {
    commitLink.href = "https://github.com/andrewkroh/http-authz-policy-middleware/commits/" + sha + "/";
}

start();
