// 📄 Dosya Yolu: /LevelUpGT/beam-patcher/beam-preview/tests/branch-ui.test.cjs
// 📌 Amac: GUI branch listesi secimini ag baglantisi olmadan dogrulamak
// 📌 Modul - JavaScript Test
// Version: 0.3.0
// Aciklama: Varsayilan ve gelismis gorunumu, eski yanitlari ve branch kimligini dogrula
// Bagimli Oldugu Katman: Tool | View | Language

"use strict";

const { test } = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const vm = require("node:vm");

const indexHtml = fs.readFileSync(path.resolve(__dirname, "../public/index.html"), "utf8");
const inlineScript = indexHtml.match(/<script>\s*([\s\S]*?)<\/script>/i)?.[1];
assert.ok(inlineScript, "BEAM preview inline controller must be present");

class FakeElement {
  constructor(id) {
    this.id = id;
    this.options = [];
    this.selectedIndex = -1;
    this.listeners = new Map();
    this.textContent = "";
    this.className = "";
    this.disabled = false;
    this._value = "";
    this.checked = false;
  }
  get value() {
    return this.options[this.selectedIndex]?.value ?? this._value;
  }
  set value(next) {
    this._value = next;
    const index = this.options.findIndex((option) => option.value === next);
    if (index >= 0) this.selectedIndex = index;
  }
  addEventListener(type, handler) {
    this.listeners.set(type, handler);
  }
  dispatchEvent(event) {
    return this.listeners.get(event.type)?.(event);
  }
  replaceChildren(...options) {
    this.options = options;
    this.selectedIndex = options.length ? 0 : -1;
    this._value = "";
  }
  append(option) {
    this.add(option);
  }
  add(option) {
    this.options.push(option);
    if (this.selectedIndex === -1) this.selectedIndex = 0;
  }
}

function boot(invoke) {
  const ids = ["result", "test", "details", "quit", "clientProfile", "sourceProject",
    "branchList", "branchSha", "branchNote", "loadBranches", "showAllBranches", "checkManifest", "manifestNote"];
  const elements = Object.fromEntries(ids.map((id) => [id, new FakeElement(id)]));
  elements.sourceProject.value = "translation";
  elements.clientProfile.options = [{ value: "hybrid-2022", text: "2022 Hybrid" }];
  elements.clientProfile.selectedIndex = 0;
  const document = { getElementById: (id) => elements[id] };
  const context = {
    document,
    window: { __TAURI__: { tauri: { invoke }, dialog: { open: async () => null } } },
    Option: class {
      constructor(text, value) { this.text = text; this.value = value; }
    },
    Event: class {
      constructor(type) { this.type = type; }
    }
  };
  vm.runInNewContext(inlineScript, context, { filename: "beam-preview/public/index.html" });
  const fire = (id, type) => elements[id].dispatchEvent(new context.Event(type));
  return { elements, fire };
}

const SHA = "af2191234a22e180fac62d179489b64f3d17a997";

test("different branch names sharing the same SHA remain independently selectable", async () => {
  const { elements, fire } = boot(async () => [
    { name: "archive/tr-2025-10-25", sha: SHA, short_sha: SHA.slice(0, 12), protected: false },
    { name: "master", sha: SHA, short_sha: SHA.slice(0, 12), protected: true }
  ]);
  await fire("loadBranches", "click");
  const items = elements.branchList.options;
  assert.deepEqual(items.map((item) => item.value), ["archive/tr-2025-10-25", "master"]);
  elements.branchList.value = "master";
  fire("branchList", "change");
  assert.match(elements.branchSha.textContent, /Seçili dal: master/);
  assert.ok(elements.branchSha.textContent.includes(SHA), "complete commit SHA is visible");
  assert.match(elements.branchSha.textContent, /korumalı dal/);
});

test("a response for an old source cannot overwrite a new source selection", async () => {
  let resolveRequest;
  const { elements, fire } = boot(() => new Promise((resolve) => { resolveRequest = resolve; }));
  const pending = fire("loadBranches", "click");
  elements.sourceProject.value = "beam";
  fire("sourceProject", "change");
  resolveRequest([{ name: "master", sha: SHA, short_sha: SHA.slice(0, 12), protected: false }]);
  await pending;
  assert.equal(elements.branchList.options.length, 1);
  assert.equal(elements.branchList.options[0].value, "");
  assert.equal(elements.branchSha.textContent, "");
  assert.match(elements.branchNote.textContent, /Yeni kaynak deposu/);
  assert.equal(elements.loadBranches.disabled, false);
});

test("a late error must not overwrite the source-switch message", async () => {
  let rejectRequest;
  const { elements, fire } = boot(() => new Promise((_resolve, reject) => { rejectRequest = reject; }));
  const pending = fire("loadBranches", "click");
  elements.sourceProject.value = "client";
  fire("sourceProject", "change");
  rejectRequest(new Error("private request failure"));
  await pending;
  assert.match(elements.branchNote.textContent, /güvenli yetkilendirme/);
});


test("standard branch mode is concise and advanced mode requests all branches", async () => {
  const calls = [];
  const response = async (command, params) => {
    assert.equal(command, "preview_list_branches");
    calls.push(params);
    const master = { name: "master", sha: SHA, short_sha: SHA.slice(0, 12), protected: false };
    const legacy = { name: "release/old-test", sha: SHA, short_sha: SHA.slice(0, 12), protected: false };
    return params.includeAll ? [master, legacy] : [master];
  };
  const { elements, fire } = boot(response);
  await fire("loadBranches", "click");
  assert.equal(calls[0].includeAll, false);
  assert.equal(elements.branchList.options.length, 1);
  elements.showAllBranches.checked = true;
  fire("showAllBranches", "change");
  assert.equal(elements.branchList.options[0].value, "");
  await fire("loadBranches", "click");
  assert.equal(calls[1].includeAll, true);
  assert.deepEqual(elements.branchList.options.map((option) => option.value),
    ["master", "release/old-test"]);
});

test("switching advanced mode invalidates a pending old-mode response", async () => {
  let resolveRequest;
  const { elements, fire } = boot(() => new Promise((resolve) => {
    resolveRequest = resolve;
  }));
  const pending = fire("loadBranches", "click");
  elements.showAllBranches.checked = true;
  fire("showAllBranches", "change");
  resolveRequest([{ name: "master", sha: SHA, short_sha: SHA.slice(0, 12), protected: false }]);
  await pending;
  assert.equal(elements.branchList.options[0].value, "");
  assert.match(elements.branchNote.textContent, /Gelişmiş görünüm seçildi/);
});

test("API failures are reported for the currently selected public source", async () => {
  const { elements, fire } = boot(async () => { throw new Error("HTTP 403"); });
  await fire("loadBranches", "click");
  assert.match(elements.branchNote.textContent, /Branch listesi alınamadı: Error: HTTP 403/);
  assert.equal(elements.branchList.options[0].value, "");
});
