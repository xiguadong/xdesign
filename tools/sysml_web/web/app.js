"use strict";

(() => {
  const $ = (id) => document.getElementById(id);
  const ui = Object.fromEntries([
    "compilerDot", "compilerVersion", "headerStatus", "fileCount", "loadProjectButton",
    "uploadButton", "newFileButton", "fileInput", "fileList", "filesEmpty", "activeFileHeading",
    "checkButton", "renderButton", "notice", "fileState", "sourceDownload", "editorPath", "sourceEditor",
    "lineNumbers", "sourceLanguage", "cursorPosition", "characterCount", "diagramTab",
    "explorerTab", "elementCount", "exportDownload", "diagramPanel", "explorerPanel",
    "viewSelect", "targetInput", "targetLabel", "viewHint", "targetOptions", "diagramViewport", "diagramStage",
    "diagramImage", "diagramEmpty", "diagramDescription", "zoomOutButton", "zoomInButton",
    "zoomLevel", "fitButton", "fitWidthButton", "svgDownload", "modelSearch", "expandAllButton", "modelTree",
    "elementDetails", "checkStatus", "reportDownload", "errorCount", "warningCount",
    "heldCount", "unevaluatedCount", "checkedFileCount", "diagnostics", "newFileDialog",
    "newFileForm", "newFileName", "newFileError", "closeDialogButton", "cancelDialogButton",
    "diagramPageButton", "backToEditorButton", "viewerRenderButton", "viewerStatus",
    "declaredViews", "declaredViewsSummary", "declaredViewList",
  ].map((id) => [id, $(id)]));

  const state = {
    files: [], activeName: null, busy: false, revision: 0, analysisRevision: -1,
    result: null, compiler: null, model: null, selectedElement: null,
    zoom: 1, diagramWidth: 0, diagramHeight: 0, diagramUrl: null,
    diagramRevision: -1, diagramView: null, diagramTarget: null,
    page: "source", viewerOrigin: null, workspaceScroll: null,
    selectedViewMode: "definitions", targetsByMode: {}, targetOptionsMode: null, targetOptionsModel: null,
  };

  function node(tag, className, text) {
    const element = document.createElement(tag);
    if (className) element.className = className;
    if (text !== undefined) element.textContent = String(text);
    return element;
  }

  function status(message) { ui.headerStatus.textContent = message; ui.viewerStatus.textContent = message; }
  function notice(message = "", tone = "") {
    ui.notice.textContent = message;
    ui.notice.className = `notice${tone ? ` ${tone}` : ""}`;
    ui.notice.hidden = !message;
  }

  function activeFile() { return state.files.find((file) => file.name === state.activeName); }
  function basename(name) { return name.split("/").pop(); }
  function normalizeName(value) {
    const name = String(value).trim().replace(/\\/g, "/");
    if (!/\.(sysml|kerml)$/.test(name)) throw new Error("File names must end in lowercase .sysml or .kerml.");
    if (!name || name.length > 512 || name.startsWith("/") || /^[A-Za-z]:/.test(name) || name.split("/").some((part) => !part || part === "." || part === "..") || /[\u0000-\u001f\u007f]/.test(name)) {
      throw new Error("Use a relative file name without empty folders or parent directory references.");
    }
    return name;
  }

  function safeUrl(value) {
    if (!value) return null;
    try {
      const url = new URL(value, window.location.href);
      return url.origin === window.location.origin && ["http:", "https:"].includes(url.protocol) ? url.href : null;
    } catch { return null; }
  }

  function setDownload(anchor, value, fileName) {
    const url = safeUrl(value);
    anchor.classList.toggle("disabled", !url);
    anchor.setAttribute("aria-disabled", String(!url));
    if (url) {
      anchor.href = url;
      if (fileName) anchor.download = fileName;
    } else anchor.removeAttribute("href");
  }

  function setBusy(busy, message) {
    state.busy = busy;
    ui.headerStatus.classList.toggle("busy", busy);
    ui.loadProjectButton.disabled = busy;
    ui.uploadButton.disabled = busy;
    ui.newFileButton.disabled = busy;
    ui.sourceEditor.disabled = busy || !activeFile();
    ui.viewSelect.disabled = busy;
    ui.targetInput.disabled = busy;
    ui.checkButton.disabled = busy || !state.files.length;
    ui.renderButton.disabled = busy || !state.files.length;
    ui.viewerRenderButton.disabled = busy || !state.files.length;
    ui.fileList.querySelectorAll(".remove-file-button").forEach((button) => { button.disabled = busy; });
    ui.declaredViewList.querySelectorAll("button").forEach((button) => { button.disabled = busy; });
    if (message) status(message);
  }

  function showCompiler(compiler) {
    if (!compiler) return;
    state.compiler = compiler;
    ui.compilerDot.classList.add("connected");
    ui.compilerVersion.textContent = `Rust · ${compiler.version || "SysML v2"}`;
    ui.compilerVersion.parentElement.title = [compiler.path, compiler.sha256 ? `SHA-256: ${compiler.sha256}` : null].filter(Boolean).join("\n");
  }

  function markChanged() {
    state.revision += 1;
    if (state.result && state.analysisRevision !== state.revision) {
      ui.checkStatus.textContent = "Source changed";
      ui.checkStatus.className = "check-status";
      status("Source changed · check or render to update results");
      ui.explorerTab.title = "Previous semantic model · source files have changed";
      if (ui.exportDownload.hasAttribute("href")) {
        ui.exportDownload.textContent = "Prior AST JSON ↓";
        ui.exportDownload.title = "Semantic AST from the previous source snapshot";
      }
      if (ui.reportDownload.hasAttribute("href")) {
        ui.reportDownload.textContent = "Prior report JSON ↓";
        ui.reportDownload.title = "Validation report from the previous source snapshot";
      }
    }
    if (state.diagramUrl) ui.diagramDescription.textContent = "Previous render · source changed";
  }

  function renderFiles() {
    ui.fileCount.textContent = state.files.length;
    ui.filesEmpty.hidden = Boolean(state.files.length);
    const fragment = document.createDocumentFragment();
    for (const file of state.files) {
      const entry = node("div", `file-entry${file.name === state.activeName ? " active" : ""}`);
      const button = node("button", `file-row${file.name === state.activeName ? " active" : ""}`);
      button.type = "button";
      button.title = file.name + (file.dirty ? "\nLocal edits" : "");
      button.setAttribute("aria-current", file.name === state.activeName ? "page" : "false");
      button.append(node("span", "file-symbol", file.name.toLowerCase().endsWith(".kerml") ? "K₂" : "S₂"));
      const info = node("span", "file-info");
      info.append(node("strong", "", basename(file.name)));
      if (file.name.includes("/")) info.append(node("small", "", file.name.slice(0, file.name.lastIndexOf("/"))));
      button.append(info);
      if (file.dirty) button.append(node("span", "dirty-dot"));
      button.addEventListener("click", () => selectFile(file.name));
      const remove = node("button", "remove-file-button", "×");
      remove.type = "button";
      remove.title = `Remove ${file.name} from this session`;
      remove.setAttribute("aria-label", `Remove ${file.name} from this session`);
      remove.disabled = state.busy;
      remove.addEventListener("click", () => removeFile(file.name));
      entry.append(button, remove);
      fragment.append(entry);
    }
    ui.fileList.replaceChildren(fragment);
    ui.checkButton.disabled = state.busy || !state.files.length;
    ui.renderButton.disabled = state.busy || !state.files.length;
    ui.viewerRenderButton.disabled = state.busy || !state.files.length;
  }

  function updateEditorDetails() {
    const file = activeFile();
    const content = ui.sourceEditor.value;
    const lineCount = content.split("\n").length;
    ui.lineNumbers.textContent = Array.from({ length: lineCount }, (_, index) => index + 1).join("\n");
    const prefix = content.slice(0, ui.sourceEditor.selectionStart);
    const line = prefix.split("\n").length;
    const column = prefix.length - prefix.lastIndexOf("\n");
    ui.cursorPosition.textContent = `Ln ${line}, Col ${column}`;
    ui.characterCount.textContent = `${content.length.toLocaleString()} characters`;
    ui.fileState.textContent = file ? file.dirty ? "Local edits" : "Loaded" : "No file";
    ui.fileState.classList.toggle("edited", Boolean(file?.dirty));
    ui.sourceLanguage.textContent = file?.name.toLowerCase().endsWith(".kerml") ? "KerML" : "SysML v2";
    ui.sourceDownload.classList.toggle("disabled", !file);
    ui.sourceDownload.setAttribute("aria-disabled", String(!file));
    syncGutter();
  }

  function syncGutter() { ui.lineNumbers.style.transform = `translateY(${-ui.sourceEditor.scrollTop}px)`; }

  function selectFile(name, line, column) {
    const file = state.files.find((item) => item.name === name);
    if (!file) return;
    state.activeName = name;
    ui.sourceEditor.value = file.content;
    ui.sourceEditor.disabled = state.busy;
    ui.activeFileHeading.textContent = basename(name);
    ui.activeFileHeading.title = name;
    ui.editorPath.textContent = name;
    ui.sourceEditor.scrollTop = 0;
    ui.sourceEditor.scrollLeft = 0;
    renderFiles();
    if (line) {
      const lines = file.content.split("\n");
      const index = Math.min(Math.max(Number(line) - 1, 0), lines.length - 1);
      const start = lines.slice(0, index).reduce((length, item) => length + item.length + 1, 0);
      const offset = start + Math.min(Math.max(Number(column || 1) - 1, 0), lines[index].length);
      ui.sourceEditor.focus();
      ui.sourceEditor.setSelectionRange(offset, Math.min(offset + 1, file.content.length));
      const lineHeight = parseFloat(getComputedStyle(ui.sourceEditor).lineHeight);
      ui.sourceEditor.scrollTop = Math.max(0, index * lineHeight - ui.sourceEditor.clientHeight / 3);
    }
    updateEditorDetails();
  }

  function removeFile(name) {
    if (state.busy) return;
    const file = state.files.find((item) => item.name === name);
    if (!file) return;
    if (file.dirty && !window.confirm(`Remove ${name} from this session? Its local edits will be discarded. Download the source first to keep them.`)) return;
    state.files = state.files.filter((item) => item.name !== name);
    markChanged();
    if (state.activeName === name) {
      state.activeName = null;
      if (state.files.length) selectFile(state.files[0].name);
      else {
        ui.sourceEditor.value = ""; ui.sourceEditor.disabled = true;
        ui.activeFileHeading.textContent = "No file selected";
        ui.editorPath.textContent = "Select or create a source file";
        updateEditorDetails();
      }
    }
    renderFiles();
    status(`${basename(name)} removed · ${state.files.length} files in workspace`);
  }

  async function jsonRequest(url, options) {
    const response = await fetch(url, options);
    let body;
    try { body = await response.json(); } catch { throw new Error(`The server returned an unreadable response (${response.status}).`); }
    if (!response.ok) throw new Error(body.error || `Request failed (${response.status}).`);
    return body;
  }

  async function loadProject() {
    if (state.busy) return;
    setBusy(true, "Loading project files…");
    notice();
    try {
      const result = await jsonRequest("/api/project");
      showCompiler(result.compiler);
      let kept = 0;
      let changed = false;
      for (const item of result.files || []) {
        const name = normalizeName(item.name);
        const content = String(item.content ?? "");
        const existing = state.files.find((file) => file.name === name);
        if (existing?.dirty) { kept += 1; continue; }
        if (existing) {
          changed ||= existing.content !== content;
          existing.content = content;
          existing.original = content;
        } else { state.files.push({ name, content, original: content, dirty: false }); changed = true; }
      }
      if (changed) markChanged();
      renderFiles();
      if (state.activeName) selectFile(state.activeName);
      else if (state.files.length) selectFile(state.files[0].name);
      status(`${state.files.length} files ready${kept ? ` · ${kept} local edits kept` : ""}`);
      if (kept) notice(`Project files loaded. Local edits in ${kept} ${kept === 1 ? "file were" : "files were"} kept.`);
      else if (!state.files.length) notice("No project model files were found. Load .sysml or .kerml files, or create a new file.");
    } catch (error) {
      notice(error.message, "error");
      status("Project could not be loaded");
    } finally { setBusy(false); }
  }

  function uniqueName(name) {
    const suffix = name.match(/\.(sysml|kerml)$/)[0];
    const stem = name.slice(0, -suffix.length);
    let index = 2;
    let candidate;
    do { candidate = `${stem}-${index++}${suffix}`; } while (state.files.some((file) => file.name === candidate));
    return candidate;
  }

  async function loadFiles(fileList) {
    const selected = [...fileList];
    if (!selected.length || state.busy) return;
    setBusy(true, "Reading source files…");
    notice();
    try {
      const entries = await Promise.all(selected.map(async (file) => ({ name: normalizeName(file.webkitRelativePath || file.name), content: await file.text() })));
      let renamed = 0;
      let firstName;
      for (const item of entries) {
        let existing = state.files.find((file) => file.name === item.name);
        if (existing?.dirty && existing.content !== item.content) { item.name = uniqueName(item.name); existing = null; renamed += 1; }
        if (existing) Object.assign(existing, { content: item.content, original: item.content, dirty: false });
        else state.files.push({ ...item, original: item.content, dirty: false });
        firstName ||= item.name;
      }
      markChanged();
      selectFile(firstName);
      status(`${entries.length} ${entries.length === 1 ? "file" : "files"} loaded · ${state.files.length} in workspace`);
      if (renamed) notice(`${renamed} uploaded ${renamed === 1 ? "file was" : "files were"} renamed to preserve existing local edits.`);
    } catch (error) { notice(error.message, "error"); status("Files could not be loaded"); }
    finally { ui.fileInput.value = ""; setBusy(false); }
  }

  function ruleCount(value) {
    if (Array.isArray(value)) return value.length;
    if (typeof value === "number") return value;
    return value == null ? "—" : String(value);
  }

  function normalizeSeverity(value) {
    const severity = String(value || "info").toLowerCase();
    if (["error", "fatal"].includes(severity)) return "error";
    if (["warning", "warn"].includes(severity)) return "warning";
    return "info";
  }

  function findDiagnosticFile(name) {
    if (!name) return null;
    const normalized = String(name).replace(/\\/g, "/");
    const exact = state.files.find((file) => file.name === normalized);
    if (exact) return exact;
    const matches = state.files.filter((file) => normalized.endsWith(`/${file.name}`) || basename(normalized) === file.name);
    return matches.length === 1 ? matches[0] : null;
  }

  function renderDiagnostics(result) {
    const diagnostics = result.diagnostics || [];
    const errors = diagnostics.filter((item) => normalizeSeverity(item.severity) === "error").length;
    const warnings = diagnostics.filter((item) => normalizeSeverity(item.severity) === "warning").length;
    ui.errorCount.textContent = errors;
    ui.warningCount.textContent = warnings;
    ui.heldCount.textContent = ruleCount(result.rules?.held);
    ui.unevaluatedCount.textContent = ruleCount(result.rules?.unevaluated);
    ui.checkedFileCount.textContent = `${state.files.length} source ${state.files.length === 1 ? "file" : "files"} checked together`;
    ui.checkStatus.textContent = result.ok ? "Checks passed" : "Needs attention";
    ui.checkStatus.className = `check-status ${result.ok ? "passed" : "failed"}`;
    const fragment = document.createDocumentFragment();
    if (!diagnostics.length) {
      const unevaluated = ruleCount(result.rules?.unevaluated);
      fragment.append(node("div", `diagnostic-empty${result.ok ? " success" : ""}`, result.ok ? `No compiler diagnostics.${Number(unevaluated) > 0 ? ` ${unevaluated} rules remain unevaluated.` : ""}` : result.error || "The compiler could not complete validation. Review the JSON report for details."));
    }
    for (const diagnostic of diagnostics) {
      const file = findDiagnosticFile(diagnostic.file);
      const row = node(file ? "button" : "div", "diagnostic-row");
      if (file) {
        row.type = "button";
        row.title = "Jump to this source location";
        row.addEventListener("click", () => selectFile(file.name, diagnostic.line || 1, diagnostic.column || 1));
      }
      const severity = normalizeSeverity(diagnostic.severity);
      row.append(node("span", `severity ${severity}`, severity));
      const content = node("div", "diagnostic-content");
      content.append(node("div", "diagnostic-message", diagnostic.message || "Compiler diagnostic"));
      if (diagnostic.file) content.append(node("div", "diagnostic-location", `${diagnostic.file}${diagnostic.line ? `:${diagnostic.line}${diagnostic.column ? `:${diagnostic.column}` : ""}` : ""}`));
      row.append(content, node("span", "diagnostic-phase", diagnostic.phase || "check"));
      fragment.append(row);
    }
    ui.diagnostics.replaceChildren(fragment);
  }

  function shortKind(kind) { return String(kind || "Element").replace(/([a-z])([A-Z])/g, "$1 $2"); }

  function showElement(element) {
    state.selectedElement = element.id;
    ui.modelTree.querySelectorAll(".selected").forEach((item) => item.classList.remove("selected"));
    for (const item of ui.modelTree.querySelectorAll("[data-element-id]")) {
      if (item.dataset.elementId === String(element.id)) item.classList.add("selected");
    }
    const fragment = document.createDocumentFragment();
    fragment.append(node("span", "eyebrow", "ELEMENT DETAILS"), node("h4", "", element.name || element.qualifiedName || "Unnamed element"));
    const details = node("dl");
    const fields = [["Kind", shortKind(element.kind)], ["Qualified name", element.qualifiedName || "—"], ["Owner", element.owner || "—"]];
    for (const [label, value] of fields) details.append(node("dt", "", label), node("dd", "", value));
    if (element.documentation) details.append(node("dt", "", "Documentation"), node("dd", "documentation", typeof element.documentation === "string" ? element.documentation : JSON.stringify(element.documentation, null, 2)));
    fragment.append(details);
    const isView = ["ViewUsage", "ViewDefinition"].includes(element.kind);
    const declaredView = (state.model?.views || []).find((item) => String(item.id) === String(element.id));
    if (isView && declaredView) {
      details.append(node("dt", "", "Rendering"), node("dd", "", declaredView.rendering || "No rendering declared"));
      details.append(node("dt", "", "Render support"), node("dd", "documentation", declaredView.canRender ? "Supported" : declaredView.reason || "This view cannot be rendered."));
    }
    if (isView || state.model?.definitions?.some((item) => String(item.id) === String(element.id))) {
      const targetButton = node("button", "button use-target-button", "Use as render target");
      targetButton.addEventListener("click", () => {
        if (isView) selectViewMode("declared", element.qualifiedName || element.name || "");
        else {
          if (ui.viewSelect.value === "declared") selectViewMode("definitions", element.qualifiedName || element.name || "");
          else ui.targetInput.value = element.qualifiedName || element.name || "";
          updateViewControls();
        }
        switchOutput("diagram"); ui.viewerRenderButton.focus();
      });
      fragment.append(targetButton);
    }
    ui.elementDetails.replaceChildren(fragment);
  }

  function renderModelTree() {
    const elements = state.model?.elements || [];
    const query = ui.modelSearch.value.trim().toLowerCase();
    if (!elements.length) { ui.modelTree.replaceChildren(node("div", "explorer-empty", "No semantic elements are available. Check a valid model to inspect its AST.")); return; }
    const byId = new Map(elements.map((element) => [String(element.id), element]));
    const children = new Map();
    for (const element of elements) {
      const owner = element.owner == null ? null : String(element.owner);
      if (owner && owner !== String(element.id) && byId.has(owner)) {
        if (!children.has(owner)) children.set(owner, []);
        children.get(owner).push(element);
      }
    }
    const matched = new Set(elements.filter((element) => !query || `${element.name || ""} ${element.qualifiedName || ""} ${element.kind || ""}`.toLowerCase().includes(query)).map((element) => String(element.id)));
    if (!matched.size) { ui.modelTree.replaceChildren(node("div", "explorer-empty", "No elements match your search.")); return; }
    const visible = new Set(matched);
    if (query) {
      for (const id of matched) {
        let current = byId.get(id);
        const visited = new Set([id]);
        while (current?.owner != null) {
          const owner = String(current.owner);
          if (visited.has(owner) || !byId.has(owner)) break;
          visited.add(owner); visible.add(owner); current = byId.get(owner);
        }
      }
    }
    const roots = [];
    const rootIds = new Set();
    const addRoot = (element) => { if (element && !rootIds.has(String(element.id))) { roots.push(element); rootIds.add(String(element.id)); } };
    for (const id of state.model.roots || []) addRoot(byId.get(String(id)));
    for (const element of elements) if (element.owner == null || !byId.has(String(element.owner)) || String(element.owner) === String(element.id)) addRoot(element);
    const tree = node("ul", "tree-list");
    const rendered = new Set();
    const appendElement = (element, parent, depth) => {
      const id = String(element.id);
      if (rendered.has(id) || !visible.has(id)) return;
      rendered.add(id);
      const item = node("li");
      const descendants = (children.get(id) || []).filter((child) => visible.has(String(child.id)) && !rendered.has(String(child.id)));
      let label;
      if (descendants.length && depth < 80) {
        const branch = node("details", "tree-branch");
        branch.open = Boolean(query) || depth < 2;
        label = node("summary");
        branch.append(label);
        const list = node("ul", "tree-list");
        for (const child of descendants) appendElement(child, list, depth + 1);
        branch.append(list); item.append(branch);
      } else { label = node("button", "tree-item"); label.type = "button"; item.append(label); }
      label.dataset.elementId = id;
      label.title = element.qualifiedName || element.name || id;
      label.append(node("span", "tree-label", element.name || element.qualifiedName || "Unnamed"), node("span", "kind-tag", shortKind(element.kind)));
      label.addEventListener("click", () => showElement(element));
      if (String(state.selectedElement) === id) label.classList.add("selected");
      parent.append(item);
    };
    for (const root of roots) appendElement(root, tree, 0);
    for (const element of elements) if (!rendered.has(String(element.id))) appendElement(element, tree, 0);
    ui.modelTree.replaceChildren(tree);
    ui.expandAllButton.textContent = "Expand all";
  }

  function renderModel(model) {
    state.model = model || { elements: [], roots: [], definitions: [], views: [] };
    state.selectedElement = null;
    ui.elementCount.textContent = state.model.elements?.length || 0;
    const viewCount = declaredViews().length;
    ui.viewSelect.querySelector('option[value="declared"]').textContent = viewCount ? `SysML view (${viewCount})` : "SysML view";
    updateViewControls({ suggest: true });
    ui.elementDetails.replaceChildren(node("span", "eyebrow", "ELEMENT DETAILS"), node("p", "", "Select an element to inspect its kind, qualified name, and documentation."));
    renderModelTree();
  }

  function switchOutput(tab) {
    const diagram = tab === "diagram";
    ui.diagramPanel.hidden = !diagram;
    ui.explorerPanel.hidden = diagram;
    for (const [button, selected] of [[ui.diagramTab, diagram], [ui.explorerTab, !diagram]]) {
      button.classList.toggle("active", selected);
      button.setAttribute("aria-selected", String(selected));
      button.tabIndex = selected ? 0 : -1;
    }
    if (diagram && state.diagramUrl) requestAnimationFrame(fitDiagramWidth);
  }

  function showPage(page) {
    const diagram = page === "diagram";
    if (diagram && state.page === "source") {
      state.viewerOrigin = document.activeElement;
      state.workspaceScroll = { x: window.scrollX, y: window.scrollY };
    }
    state.page = page;
    document.body.classList.toggle("diagram-page", diagram);
    ui.backToEditorButton.hidden = !diagram;
    ui.viewerRenderButton.hidden = !diagram;
    ui.viewerStatus.hidden = !diagram;
    if (diagram) switchOutput("diagram");
    requestAnimationFrame(() => {
      fitDiagramWidth();
      if (diagram) ui.backToEditorButton.focus();
      else {
        (state.viewerOrigin?.isConnected ? state.viewerOrigin : ui.renderButton).focus({ preventScroll: true });
        if (state.workspaceScroll) window.scrollTo(state.workspaceScroll.x, state.workspaceScroll.y);
      }
    });
  }

  function clearDiagram(message = "No diagram yet") {
    state.diagramUrl = null; state.diagramWidth = 0; state.diagramHeight = 0;
    ui.diagramImage.removeAttribute("src");
    ui.diagramStage.hidden = true; ui.diagramEmpty.hidden = false;
    ui.diagramViewport.classList.remove("has-diagram");
    ui.diagramDescription.textContent = message;
    ui.zoomLevel.textContent = "100%";
    for (const button of [ui.zoomOutButton, ui.zoomInButton, ui.fitButton, ui.fitWidthButton]) button.disabled = true;
    setDownload(ui.svgDownload, null);
  }

  function setZoom(value, centered = true) {
    if (state.page !== "diagram" || !state.diagramWidth || !state.diagramHeight || !ui.diagramViewport.clientWidth || !ui.diagramViewport.clientHeight) return;
    const viewport = ui.diagramViewport;
    const oldWidth = state.diagramWidth * state.zoom;
    const oldHeight = state.diagramHeight * state.zoom;
    const centerX = (viewport.scrollLeft + viewport.clientWidth / 2 - ui.diagramImage.offsetLeft) / Math.max(oldWidth, 1);
    const centerY = (viewport.scrollTop + viewport.clientHeight / 2 - ui.diagramImage.offsetTop) / Math.max(oldHeight, 1);
    state.zoom = Math.max(0.05, Math.min(8, value));
    const width = state.diagramWidth * state.zoom;
    const height = state.diagramHeight * state.zoom;
    ui.diagramImage.style.width = `${width}px`;
    ui.diagramImage.style.height = `${height}px`;
    ui.diagramStage.style.width = `${Math.max(viewport.clientWidth, width + 48)}px`;
    ui.diagramStage.style.height = `${Math.max(viewport.clientHeight, height + 48)}px`;
    ui.zoomLevel.textContent = `${Math.round(state.zoom * 100)}%`;
    if (centered) {
      viewport.scrollLeft = Math.max(0, ui.diagramImage.offsetLeft + centerX * width - viewport.clientWidth / 2);
      viewport.scrollTop = Math.max(0, ui.diagramImage.offsetTop + centerY * height - viewport.clientHeight / 2);
    }
  }

  function fitDiagram() {
    if (state.page !== "diagram" || !state.diagramWidth || !state.diagramHeight || ui.diagramPanel.hidden) return;
    const availableWidth = Math.max(1, ui.diagramViewport.clientWidth - 48);
    const availableHeight = Math.max(1, ui.diagramViewport.clientHeight - 48);
    setZoom(Math.min(availableWidth / state.diagramWidth, availableHeight / state.diagramHeight), false);
    ui.diagramViewport.scrollLeft = 0; ui.diagramViewport.scrollTop = 0;
  }

  function fitDiagramWidth() {
    if (state.page !== "diagram" || !state.diagramWidth || !state.diagramHeight || ui.diagramPanel.hidden) return;
    const width = ui.diagramViewport.clientWidth;
    if (!width || !ui.diagramViewport.clientHeight) return;
    setZoom(Math.min(1.5, Math.max(1, width - 48) / state.diagramWidth), false);
    ui.diagramViewport.scrollLeft = 0; ui.diagramViewport.scrollTop = 0;
  }

  function declaredViews() { return Array.isArray(state.model?.views) ? state.model.views : []; }

  function selectedDeclaredView() {
    const target = ui.targetInput.value.trim();
    const exact = declaredViews().find((view) => view.qualifiedName === target);
    if (exact) return exact;
    const matches = declaredViews().filter((view) => view.name === target);
    return matches.length === 1 ? matches[0] : null;
  }

  function selectViewMode(mode, target) {
    if (mode !== state.selectedViewMode) {
      state.targetsByMode[state.selectedViewMode] = ui.targetInput.value;
      state.selectedViewMode = mode;
      ui.targetInput.value = state.targetsByMode[mode] || "";
    }
    ui.viewSelect.value = mode;
    if (target !== undefined) ui.targetInput.value = target;
    updateViewControls({ suggest: true });
  }

  function renderTargetChoices() {
    const declared = ui.viewSelect.value === "declared";
    const views = declaredViews();
    if (state.targetOptionsMode !== ui.viewSelect.value || state.targetOptionsModel !== state.model) {
      const options = document.createDocumentFragment();
      for (const target of declared ? views : state.model?.definitions || []) {
        const option = node("option");
        option.value = target.qualifiedName || target.name || "";
        option.label = declared ? target.canRender ? target.rendering || "Supported model view" : `Unsupported: ${target.reason || "No supported rendering"}` : shortKind(target.kind);
        options.append(option);
      }
      ui.targetOptions.replaceChildren(options);
      state.targetOptionsMode = ui.viewSelect.value;
      state.targetOptionsModel = state.model;
    }
    ui.declaredViews.hidden = !declared;
    if (!declared) return;
    const supported = views.filter((view) => view.canRender).length;
    ui.declaredViewsSummary.textContent = views.length ? `Declared model views · ${supported} supported · ${views.length - supported} unsupported` : "Declared model views";
    const list = document.createDocumentFragment();
    if (!views.length) list.append(node("p", "declared-views-empty", "No declared views are available. Check a model that defines views and renderings."));
    for (const view of views) {
      const name = view.qualifiedName || view.name || "Unnamed view";
      const selected = selectedDeclaredView()?.id === view.id;
      const choice = node("button", `declared-view-choice${view.canRender ? "" : " unsupported"}${selected ? " selected" : ""}`);
      choice.type = "button";
      choice.disabled = state.busy;
      choice.setAttribute("aria-pressed", String(selected));
      choice.title = [name, view.rendering, view.canRender ? "Supported" : view.reason || "No supported rendering"].filter(Boolean).join("\n");
      choice.append(node("span", "declared-view-name", name), node("span", "declared-view-support", view.canRender ? view.rendering || "Supported" : "Unsupported"));
      choice.addEventListener("click", () => { ui.targetInput.value = name; updateViewControls(); ui.viewerRenderButton.focus(); });
      list.append(choice);
    }
    ui.declaredViewList.replaceChildren(list);
  }

  function updateViewControls({ suggest = false } = {}) {
    const view = ui.viewSelect.value;
    if (view === "declared" && suggest && !ui.targetInput.value.trim()) {
      const firstSupported = declaredViews().find((item) => item.canRender);
      if (firstSupported) ui.targetInput.value = firstSupported.qualifiedName || firstSupported.name || "";
    }
    const required = ["internal", "sequence", "declared"].includes(view);
    ui.targetLabel.textContent = required ? "Target (required)" : "Target";
    ui.targetInput.required = required;
    ui.targetInput.setAttribute("aria-required", String(required));
    const hints = {
      definitions: ["All definitions", "Target is optional. Show all definitions or focus on one."],
      tree: ["Whole model", "Target is optional. Explore containment in the semantic model."],
      internal: ["Required: part definition", "Internal structure requires a definition target. Check first to browse available definitions."],
      sequence: ["Required: action definition", "Sequence requires a definition target with actions or interactions."],
      declared: ["Required: declared SysML view", "Select a model-defined view. Its expose and render declarations determine the model scope and diagram."],
    };
    ui.targetInput.placeholder = hints[view][0];
    ui.viewHint.textContent = hints[view][1];
    const declaredView = view === "declared" ? selectedDeclaredView() : null;
    if (declaredView) {
      ui.viewHint.textContent += declaredView.canRender ? ` Rendering: ${declaredView.rendering || "model declaration"}.` : ` Unsupported: ${declaredView.reason || "This view has no supported rendering."}`;
    } else if (view === "declared" && ui.targetInput.value.trim()) ui.viewHint.textContent += " This target is not listed in the current model export.";
    ui.viewHint.classList.toggle("view-warning", Boolean(declaredView && !declaredView.canRender));
    renderTargetChoices();
    if (state.diagramUrl && (view !== state.diagramView || ui.targetInput.value.trim() !== state.diagramTarget)) {
      ui.diagramDescription.textContent = "Previous render · view or target changed";
    } else if (state.diagramUrl && state.diagramRevision === state.revision) {
      ui.diagramDescription.textContent = `${shortKind(state.diagramView)} view`;
    }
  }

  function showDiagram(diagram) {
    const url = safeUrl(diagram?.url);
    clearDiagram(diagram ? "Loading diagram…" : "Rendering did not produce a diagram");
    if (!url) return;
    state.diagramUrl = url;
    switchOutput("diagram");
    ui.diagramImage.onload = () => {
      if (state.diagramUrl !== url) return;
      state.diagramWidth = ui.diagramImage.naturalWidth || 1000;
      state.diagramHeight = ui.diagramImage.naturalHeight || 700;
      ui.diagramStage.hidden = false; ui.diagramEmpty.hidden = true;
      ui.diagramViewport.classList.add("has-diagram");
      ui.diagramDescription.textContent = `${shortKind(diagram.view || ui.viewSelect.value)} view`;
      for (const button of [ui.zoomOutButton, ui.zoomInButton, ui.fitButton, ui.fitWidthButton]) button.disabled = false;
      setDownload(ui.svgDownload, url, `model-${diagram.view || ui.viewSelect.value}.svg`);
      requestAnimationFrame(fitDiagramWidth);
    };
    ui.diagramImage.onerror = () => {
      if (state.diagramUrl !== url) return;
      clearDiagram("Diagram could not be loaded");
      notice("The diagram could not be loaded. Review the report and try rendering again.", "error");
    };
    ui.diagramImage.src = url;
  }

  async function analyze(operation) {
    if (state.busy || !state.files.length) return;
    if (operation === "render" && ["internal", "sequence", "declared"].includes(ui.viewSelect.value) && !ui.targetInput.value.trim()) {
      notice(ui.viewSelect.value === "declared" ? "Choose a declared SysML view. Check the model first to discover its views and renderings." : "Choose a definition target for this view. Check the model first to populate the target suggestions.", "warning");
      ui.targetInput.focus();
      return;
    }
    setBusy(true, operation === "render" ? "Checking and rendering model…" : "Checking all loaded files…");
    notice();
    const revision = state.revision;
    try {
      const result = await jsonRequest("/api/analyze", {
        method: "POST", headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ files: state.files.map(({ name, content }) => ({ name, content })), operation, view: ui.viewSelect.value, target: ui.targetInput.value.trim() }),
      });
      state.result = result; state.analysisRevision = revision;
      showCompiler(result.compiler);
      renderDiagnostics(result);
      if (result.model?.elements?.length || !state.model?.elements?.length) {
        renderModel(result.model);
        ui.explorerTab.title = "Semantic model from the current analysis";
      } else ui.explorerTab.title = "Previous semantic model · current analysis produced no model";
      setDownload(ui.reportDownload, result.artifacts?.report, "sysml-report.json");
      ui.reportDownload.textContent = "Report JSON ↓";
      ui.reportDownload.title = "Validation report from the current analysis";
      if (result.artifacts?.export) {
        setDownload(ui.exportDownload, result.artifacts.export, "sysml-ast.json");
        ui.exportDownload.textContent = "AST JSON ↓";
        ui.exportDownload.title = "Semantic AST from the current analysis";
      } else if (ui.exportDownload.hasAttribute("href")) {
        ui.exportDownload.textContent = "Prior AST JSON ↓";
        ui.exportDownload.title = "Previous export · current analysis produced no semantic AST";
      }
      if (operation === "render" && result.diagram) showDiagram(result.diagram);
      else if (operation === "render" && state.diagramUrl) ui.diagramDescription.textContent = "Previous render · current render failed";
      else if (operation === "render") clearDiagram("Rendering did not produce a diagram");
      else if (state.diagramUrl && revision !== state.diagramRevision) ui.diagramDescription.textContent = "Previous render · render again to update";
      if (result.diagram) {
        state.diagramRevision = revision;
        state.diagramView = result.diagram.view || ui.viewSelect.value;
        state.diagramTarget = ui.targetInput.value.trim();
      }
      if (result.error) notice(result.error, result.ok ? "warning" : "error");
      const viewCount = declaredViews().length;
      status(result.ok ? operation === "render" ? result.diagram ? "Model checked and rendered" : "Model checked · no diagram produced" : `Compiler checks passed${viewCount ? ` · ${viewCount} declared ${viewCount === 1 ? "view" : "views"}` : ""}` : "Model needs attention · review diagnostics");
    } catch (error) {
      notice(error.message, "error"); status("Analysis could not be completed");
      ui.checkStatus.textContent = "Request failed"; ui.checkStatus.className = "check-status failed";
    } finally { setBusy(false); }
  }

  ui.sourceEditor.addEventListener("input", () => {
    const file = activeFile();
    if (!file) return;
    file.content = ui.sourceEditor.value; file.dirty = file.content !== file.original;
    markChanged(); updateEditorDetails(); renderFiles();
  });
  ui.sourceEditor.addEventListener("scroll", syncGutter);
  for (const event of ["click", "keyup", "select"]) ui.sourceEditor.addEventListener(event, updateEditorDetails);
  ui.sourceEditor.addEventListener("keydown", (event) => {
    if (event.key !== "Tab" || event.ctrlKey || event.metaKey || event.altKey || event.shiftKey) return;
    event.preventDefault();
    const start = ui.sourceEditor.selectionStart; const end = ui.sourceEditor.selectionEnd;
    ui.sourceEditor.setRangeText("    ", start, end, "end");
    ui.sourceEditor.dispatchEvent(new Event("input"));
  });
  document.addEventListener("keydown", (event) => {
    if (event.key === "Escape" && state.page === "diagram") { event.preventDefault(); showPage("source"); return; }
    if ((event.ctrlKey || event.metaKey) && event.key === "Enter" && !ui.newFileDialog.open) { event.preventDefault(); analyze("check"); }
  });
  ui.loadProjectButton.addEventListener("click", loadProject);
  ui.uploadButton.addEventListener("click", () => ui.fileInput.click());
  ui.fileInput.addEventListener("change", () => loadFiles(ui.fileInput.files));
  ui.checkButton.addEventListener("click", () => analyze("check"));
  ui.renderButton.addEventListener("click", () => { showPage("diagram"); analyze("render"); });
  ui.viewerRenderButton.addEventListener("click", () => analyze("render"));
  ui.diagramPageButton.addEventListener("click", () => showPage("diagram"));
  ui.backToEditorButton.addEventListener("click", () => showPage("source"));
  ui.viewSelect.addEventListener("change", () => selectViewMode(ui.viewSelect.value));
  ui.targetInput.addEventListener("input", () => updateViewControls());
  ui.newFileButton.addEventListener("click", () => {
    ui.newFileError.hidden = true; ui.newFileName.value = "model.sysml";
    ui.newFileDialog.showModal(); ui.newFileName.select();
  });
  for (const button of [ui.closeDialogButton, ui.cancelDialogButton]) button.addEventListener("click", () => ui.newFileDialog.close());
  ui.newFileForm.addEventListener("submit", (event) => {
    event.preventDefault();
    try {
      const name = normalizeName(ui.newFileName.value);
      if (state.files.some((file) => file.name === name)) throw new Error("A file with this name is already loaded.");
      const content = name.toLowerCase().endsWith(".kerml") ? "package NewModel {\n    class System {\n    }\n}\n" : "package NewModel {\n    part def System {\n    }\n}\n";
      state.files.push({ name, content, original: "", dirty: true });
      markChanged(); selectFile(name); ui.newFileDialog.close(); ui.sourceEditor.focus();
      status("New file created · edit its source and check the model");
    } catch (error) { ui.newFileError.textContent = error.message; ui.newFileError.hidden = false; }
  });
  ui.sourceDownload.addEventListener("click", (event) => {
    const file = activeFile();
    if (!file) { event.preventDefault(); return; }
    const url = URL.createObjectURL(new Blob([file.content], { type: "text/plain;charset=utf-8" }));
    ui.sourceDownload.href = url; ui.sourceDownload.download = basename(file.name);
    window.setTimeout(() => URL.revokeObjectURL(url), 10000);
  });
  ui.diagramTab.addEventListener("click", () => switchOutput("diagram"));
  ui.explorerTab.addEventListener("click", () => switchOutput("explorer"));
  for (const tab of [ui.diagramTab, ui.explorerTab]) tab.addEventListener("keydown", (event) => {
    if (!["ArrowLeft", "ArrowRight", "Home", "End"].includes(event.key)) return;
    event.preventDefault();
    const next = event.key === "Home" ? ui.diagramTab : event.key === "End" ? ui.explorerTab : tab === ui.diagramTab ? ui.explorerTab : ui.diagramTab;
    switchOutput(next === ui.diagramTab ? "diagram" : "explorer"); next.focus();
  });
  ui.modelSearch.addEventListener("input", renderModelTree);
  ui.expandAllButton.addEventListener("click", () => {
    const details = [...ui.modelTree.querySelectorAll("details")];
    const open = details.some((item) => !item.open);
    details.forEach((item) => { item.open = open; });
    ui.expandAllButton.textContent = open ? "Collapse all" : "Expand all";
  });
  ui.zoomOutButton.addEventListener("click", () => setZoom(state.zoom / 1.25));
  ui.zoomInButton.addEventListener("click", () => setZoom(state.zoom * 1.25));
  ui.fitButton.addEventListener("click", fitDiagram);
  ui.fitWidthButton.addEventListener("click", fitDiagramWidth);
  ui.diagramViewport.addEventListener("wheel", (event) => {
    if (!state.diagramUrl || !(event.ctrlKey || event.metaKey)) return;
    event.preventDefault(); setZoom(state.zoom * (event.deltaY < 0 ? 1.1 : 1 / 1.1));
  }, { passive: false });
  let pan = null;
  ui.diagramViewport.addEventListener("pointerdown", (event) => {
    if (!state.diagramUrl || event.button !== 0 || event.pointerType === "touch") return;
    pan = { x: event.clientX, y: event.clientY, left: ui.diagramViewport.scrollLeft, top: ui.diagramViewport.scrollTop };
    ui.diagramViewport.setPointerCapture(event.pointerId); ui.diagramViewport.classList.add("panning");
  });
  ui.diagramViewport.addEventListener("pointermove", (event) => {
    if (!pan) return;
    ui.diagramViewport.scrollLeft = pan.left - event.clientX + pan.x;
    ui.diagramViewport.scrollTop = pan.top - event.clientY + pan.y;
  });
  const endPan = () => { pan = null; ui.diagramViewport.classList.remove("panning"); };
  for (const event of ["pointerup", "pointercancel", "lostpointercapture"]) ui.diagramViewport.addEventListener(event, endPan);
  if (typeof ResizeObserver !== "undefined") new ResizeObserver(() => { if (state.diagramUrl && state.diagramWidth) setZoom(state.zoom, false); }).observe(ui.diagramViewport);
  window.addEventListener("beforeunload", (event) => {
    if (state.files.some((file) => file.dirty)) { event.preventDefault(); event.returnValue = ""; }
  });
  loadProject();
})();
