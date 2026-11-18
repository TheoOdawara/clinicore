const text = {
  button: "Exportar para PDF",
  heading: "Exportar para PDF",
  thisPage: "Esta página",
  thisSection: "Esta seção",
  allDocs: "Toda a documentação",
  expand: "Mostrar as páginas de",
  selected: (count) => (count === 1 ? "1 página selecionada" : `${count} páginas selecionadas`),
  pages: (count) => `${count} páginas`,
  cancel: "Cancelar",
  export: "Exportar PDF",
  preparing: (done, total) => `Preparando ${done} de ${total}…`,
  contents: "Sumário",
  pageFailed: (title, status) => `“${title}” não carregou (HTTP ${status}).`,
  noArticle: (title) => `“${title}” não tem conteúdo para exportar.`,
  imageFailed: (title, path) => `Uma imagem de “${title}” não carregou: ${path}`,
};

const header = document.querySelector(".md-header__inner");
const primary = document.querySelector("nav.md-nav--primary > ul.md-nav__list");
const logoLink = document.querySelector('a.md-logo[data-md-component="logo"]');

const root = new URL("./", new URL(logoLink?.getAttribute("href") ?? "./", location.href));
const relative = (url) => decodeURIComponent(url.pathname).slice(decodeURIComponent(root.pathname).length);
const pageOf = (url) => {
  const clean = new URL(url);
  clean.hash = "";
  clean.search = "";
  clean.pathname = clean.pathname.replace(/index\.html$/, "");
  return clean;
};
const siteName =
  document.querySelector(".md-header__topic:first-child .md-ellipsis")?.textContent.trim() || document.title;

const readTree = (list, seen) => {
  const nodes = [];
  for (const item of list.children) {
    if (!item.matches("li.md-nav__item")) continue;
    if (item.classList.contains("md-nav__item--nested")) {
      const title = item.querySelector(":scope > .md-nav__link .md-ellipsis")?.textContent.trim() ?? "";
      const nested = item.querySelector(":scope > nav.md-nav > ul.md-nav__list");
      const children = nested ? readTree(nested, seen) : [];
      const index = item.querySelector(":scope > a.md-nav__link[href], :scope > .md-nav__container > a.md-nav__link[href]");
      const indexPage = index && readPage(index, seen);
      if (indexPage) children.unshift(indexPage);
      if (children.length > 0) nodes.push({ title, children });
      continue;
    }
    const link = item.querySelector(":scope > a.md-nav__link[href]");
    const page = link && readPage(link, seen);
    if (page) nodes.push(page);
  }
  return nodes;
};

const readPage = (link, seen) => {
  if (link.getAttribute("href").startsWith("#")) return null;
  const url = pageOf(new URL(link.getAttribute("href"), location.href));
  if (url.origin !== root.origin || !url.pathname.startsWith(root.pathname)) return null;
  const path = relative(url);
  if (seen.has(path)) return null;
  seen.add(path);
  return { title: link.querySelector(".md-ellipsis")?.textContent.trim() || link.textContent.trim(), path, url };
};

const pagesUnder = (node) => (node.children ? node.children.flatMap(pagesUnder) : [node]);

const tree = primary ? readTree(primary, new Set()) : [];
const order = tree.flatMap(pagesUnder);
const currentPath = relative(pageOf(location.href));
const storageKey = `pdf-export:${root.pathname}`;

const sectionOf = (nodes, trail = []) => {
  for (const node of nodes) {
    if (!node.children) continue;
    if (!pagesUnder(node).some((page) => page.path === currentPath)) continue;
    return sectionOf(node.children, [...trail, node]) ?? node;
  }
  return trail.at(-1) ?? null;
};
const section = sectionOf(tree);

const shortcuts = {
  page: () => order.filter((page) => page.path === currentPath),
  ...(section && { section: () => pagesUnder(section) }),
  all: () => order,
};

const readStored = () => {
  try {
    const stored = JSON.parse(localStorage.getItem(storageKey) ?? "[]");
    return Array.isArray(stored) ? order.filter((page) => stored.includes(page.path)) : [];
  } catch {
    return [];
  }
};
const store = (paths) => {
  try {
    localStorage.setItem(storageKey, JSON.stringify(paths));
  } catch {
    // shortcut: the selection is not remembered when storage is blocked; keep it in the URL if readers need that
    return;
  }
};

const selected = new Set();

const dialog = document.createElement("dialog");
dialog.className = "pdf-export-dialog";
dialog.setAttribute("aria-labelledby", "pdf-export-heading");
dialog.innerHTML = `
  <h2 class="pdf-export-dialog__heading" id="pdf-export-heading">${text.heading}</h2>
  <div class="pdf-export-dialog__shortcuts" role="group">
    <button class="pdf-export-dialog__chip" type="button" data-shortcut="page">${text.thisPage}</button>
    <button class="pdf-export-dialog__chip" type="button" data-shortcut="section" hidden></button>
    <button class="pdf-export-dialog__chip" type="button" data-shortcut="all">${text.allDocs}</button>
  </div>
  <div class="pdf-export-dialog__tree"></div>
  <p class="pdf-export-dialog__status" role="status" aria-live="polite"></p>
  <div class="pdf-export-dialog__actions">
    <button class="pdf-export-dialog__button" type="button" data-action="cancel">${text.cancel}</button>
    <button class="pdf-export-dialog__button pdf-export-dialog__button--primary" type="button" data-action="export">${text.export}</button>
  </div>`;
if (section) {
  const chip = dialog.querySelector('[data-shortcut="section"]');
  chip.textContent = `${text.thisSection}: ${section.title}`;
  chip.hidden = false;
}
const cancelButton = dialog.querySelector('[data-action="cancel"]');
const exportButton = dialog.querySelector('[data-action="export"]');
const treeView = dialog.querySelector(".pdf-export-dialog__tree");
const status = dialog.querySelector(".pdf-export-dialog__status");

const checkboxes = [];

const renderTree = (nodes, depth) => {
  const list = document.createElement("ul");
  list.className = "pdf-export-tree";
  for (const node of nodes) {
    const item = document.createElement("li");
    const row = document.createElement("div");
    row.className = "pdf-export-tree__row";
    row.style.setProperty("--depth", depth);
    const label = document.createElement("label");
    label.className = "pdf-export-tree__label";
    const checkbox = document.createElement("input");
    checkbox.type = "checkbox";
    const title = document.createElement("span");
    title.textContent = node.title;
    label.append(checkbox, title);
    if (node.children) {
      const pages = pagesUnder(node);
      const toggle = document.createElement("button");
      toggle.type = "button";
      toggle.className = "pdf-export-tree__toggle";
      toggle.setAttribute("aria-label", `${text.expand} ${node.title}`);
      const children = renderTree(node.children, depth + 1);
      const expand = (open) => {
        toggle.setAttribute("aria-expanded", String(open));
        children.hidden = !open;
      };
      expand(pages.some((page) => page.path === currentPath));
      toggle.addEventListener("click", () => expand(children.hidden));
      checkbox.addEventListener("change", () => {
        const every = pages.every((page) => selected.has(page.path));
        for (const page of pages) every ? selected.delete(page.path) : selected.add(page.path);
        sync();
      });
      checkboxes.push({ checkbox, pages, expand, isOpen: () => !children.hidden });
      row.append(toggle, label);
      item.append(row, children);
    } else {
      checkbox.addEventListener("change", () => {
        checkbox.checked ? selected.add(node.path) : selected.delete(node.path);
        sync();
      });
      checkboxes.push({ checkbox, pages: [node] });
      row.append(label);
      item.append(row);
    }
    list.append(item);
  }
  return list;
};
treeView.append(renderTree(tree, 0));

const sameSet = (pages) => pages.length === selected.size && pages.every((page) => selected.has(page.path));

const sync = () => {
  for (const entry of checkboxes) {
    const count = entry.pages.filter((page) => selected.has(page.path)).length;
    entry.checkbox.checked = count === entry.pages.length;
    entry.checkbox.indeterminate = count > 0 && count < entry.pages.length;
    if (entry.expand && entry.checkbox.indeterminate && !entry.isOpen()) entry.expand(true);
  }
  for (const [name, pick] of Object.entries(shortcuts)) {
    dialog.querySelector(`[data-shortcut="${name}"]`).setAttribute("aria-pressed", String(sameSet(pick())));
  }
  exportButton.disabled = selected.size === 0;
  showStatus(text.selected(selected.size));
  store([...selected]);
};

const showStatus = (message, failed = false) => {
  status.textContent = message;
  status.classList.toggle("pdf-export-dialog__status--error", failed);
};

const choose = (pages) => {
  selected.clear();
  for (const page of pages) selected.add(page.path);
  sync();
};

dialog.addEventListener("click", (event) => {
  if (event.target === dialog) dialog.close();
  const shortcut = event.target.closest("[data-shortcut]")?.dataset.shortcut;
  if (shortcut) choose(shortcuts[shortcut]());
  const action = event.target.closest("[data-action]")?.dataset.action;
  if (action === "cancel") dialog.close();
  if (action === "export") exportSelection();
});

let running = null;
dialog.addEventListener("close", () => running?.abort());

const setBusy = (busy) => {
  if (busy) cancelButton.focus();
  dialog.querySelectorAll("button, input").forEach((control) => (control.disabled = busy));
  cancelButton.disabled = false;
  if (busy) return;
  exportButton.textContent = text.export;
  exportButton.disabled = selected.size === 0;
};

const fetchPage = async (page, signal) => {
  const response = await fetch(page.url, { signal });
  if (!response.ok) throw new Error(text.pageFailed(page.title, response.status));
  return new DOMParser().parseFromString(await response.text(), "text/html");
};

const abortion = (signal) =>
  new Promise((_, reject) => {
    const fail = () => reject(new DOMException("closed", "AbortError"));
    if (signal.aborted) fail();
    signal.addEventListener("abort", fail, { once: true });
  });

const loaded = (element) =>
  new Promise((resolve) => {
    element.addEventListener("load", resolve, { once: true });
    element.addEventListener("error", resolve, { once: true });
  });

const prepare = (article, page, anchors) => {
  const prefix = anchors.get(page.path);
  article.querySelectorAll(".headerlink, .md-content__button, .md-clipboard").forEach((element) => element.remove());
  article.querySelectorAll("details").forEach((details) => (details.open = true));
  article.querySelectorAll("[id]").forEach((element) => (element.id = `${prefix}--${element.id}`));
  article.querySelectorAll("label[for]").forEach((label) => (label.htmlFor = `${prefix}--${label.htmlFor}`));
  article.querySelectorAll("[src]").forEach((element) =>
    element.setAttribute("src", new URL(element.getAttribute("src"), page.url).href),
  );
  article.querySelectorAll("img").forEach((image) => (image.loading = "eager"));
  for (const link of article.querySelectorAll("a[href]")) {
    const target = new URL(link.getAttribute("href"), page.url);
    if (target.origin !== root.origin) continue;
    const hash = decodeURIComponent(target.hash.slice(1));
    const anchor = anchors.get(relative(pageOf(target)));
    if (anchor) link.setAttribute("href", hash ? `#${anchor}--${hash}` : `#${anchor}`);
    else link.removeAttribute("href");
  }
};

const scopeName = (chosen) => {
  if (chosen.length === order.length) return null;
  if (chosen.length === 1) return chosen[0].title;
  const folders = [];
  const walk = (nodes) => nodes.forEach((node) => node.children && (folders.push(node), walk(node.children)));
  walk(tree);
  return folders.find((folder) => sameSet(pagesUnder(folder)))?.title ?? text.pages(chosen.length);
};

const cover = (scope) => {
  const page = document.createElement("section");
  page.className = "pdf-export__cover";
  const logo = document.querySelector(".md-header .md-logo img");
  if (logo) {
    const image = document.createElement("img");
    image.src = logo.currentSrc || logo.src;
    image.alt = "";
    page.append(image);
  }
  const title = document.createElement("h1");
  title.textContent = siteName;
  page.append(title);
  if (scope) {
    const line = document.createElement("p");
    line.className = "pdf-export__scope";
    line.textContent = scope;
    page.append(line);
  }
  const date = document.createElement("p");
  date.className = "pdf-export__date";
  date.textContent = new Intl.DateTimeFormat(document.documentElement.lang || undefined, { dateStyle: "long" }).format(new Date());
  page.append(date);
  return page;
};

const contents = (entries) => {
  const page = document.createElement("nav");
  page.className = "pdf-export__contents md-typeset";
  const heading = document.createElement("h1");
  heading.textContent = text.contents;
  const list = document.createElement("ol");
  for (const entry of entries) {
    const item = document.createElement("li");
    item.style.setProperty("--depth", entry.depth);
    const link = document.createElement("a");
    link.href = `#${entry.anchor}`;
    link.textContent = entry.title;
    item.append(link);
    list.append(item);
  }
  page.append(heading, list);
  return page;
};

let pendingRestore = null;

const printAndRestore = (container, title) => {
  const html = document.documentElement;
  const body = document.body;
  const scheme = body.getAttribute("data-md-color-scheme");
  const pageTitle = document.title;
  const pageHeader = document.createElement("style");
  pageHeader.textContent = `@page { @top-left { content: ${JSON.stringify(siteName)}; } }`;
  document.head.append(pageHeader);
  html.classList.add("pdf-exporting");
  body.setAttribute("data-md-color-scheme", "default");
  document.title = title;
  const restore = () => {
    removeEventListener("afterprint", restore);
    pendingRestore = null;
    html.classList.remove("pdf-exporting");
    if (scheme === null) body.removeAttribute("data-md-color-scheme");
    else body.setAttribute("data-md-color-scheme", scheme);
    document.title = pageTitle;
    pageHeader.remove();
    container.remove();
  };
  pendingRestore = restore;
  addEventListener("afterprint", restore, { once: true });
  window.print();
};

const exportSelection = async () => {
  const chosen = order.filter((page) => selected.has(page.path));
  if (chosen.length === 0) return;
  pendingRestore?.();
  const controller = new AbortController();
  running = controller;
  setBusy(true);
  const container = document.createElement("div");
  container.className = "pdf-export";
  try {
    const anchors = new Map(chosen.map((page, index) => [page.path, `pdf-export-page-${index + 1}`]));
    const sections = [];
    for (const [index, page] of chosen.entries()) {
      exportButton.textContent = text.preparing(index + 1, chosen.length);
      showStatus(text.preparing(index + 1, chosen.length));
      const source = await fetchPage(page, controller.signal);
      const article = source.querySelector("article.md-content__inner");
      if (!article) throw new Error(text.noArticle(page.title));
      prepare(article, page, anchors);
      const section = document.createElement("section");
      section.className = "pdf-export__page";
      section.id = anchors.get(page.path);
      section.dataset.title = page.title;
      section.append(document.adoptNode(article));
      sections.push(section);
    }
    const scope = scopeName(chosen);
    if (chosen.length > 1) {
      const entries = sections.map((section, index) => ({
        anchor: section.id,
        title: section.querySelector("h1")?.textContent.trim() || chosen[index].title,
        depth: chosen[index].path.split("/").filter(Boolean).length,
      }));
      container.append(cover(scope), contents(entries));
    }
    container.append(...sections);
    document.body.append(container);
    const images = [...container.querySelectorAll("img")].filter((image) => !image.complete).map(loaded);
    await Promise.race([Promise.all([...images, document.fonts.ready]), abortion(controller.signal)]);
    for (const image of container.querySelectorAll("img")) {
      if (image.naturalWidth > 0) continue;
      const title = image.closest(".pdf-export__page")?.dataset.title ?? siteName;
      throw new Error(text.imageFailed(title, relative(new URL(image.src))));
    }
    running = null;
    setBusy(false);
    dialog.close();
    printAndRestore(container, scope ? `${siteName} - ${scope}` : siteName);
  } catch (error) {
    container.remove();
    if (running !== controller) return;
    setBusy(false);
    if (error.name !== "AbortError") {
      showStatus(error.message, true);
      exportButton.focus();
    }
  } finally {
    if (running === controller) running = null;
  }
};

if (header && order.length > 0) {
  const button = document.createElement("button");
  button.type = "button";
  button.className = "md-header__button md-icon pdf-export-button";
  button.title = text.button;
  button.setAttribute("aria-label", text.button);
  button.innerHTML = `<svg viewBox="0 0 24 24" aria-hidden="true" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 3v12"/><path d="m7 10 5 5 5-5"/><path d="M5 21h14"/></svg>`;
  button.addEventListener("click", () => {
    const stored = readStored();
    running?.abort();
    running = null;
    setBusy(false);
    choose(stored.length > 0 ? stored : shortcuts.page());
    dialog.showModal();
  });
  const anchor = header.querySelector('[data-md-component="palette"], label[for="__search"], .md-header__source');
  anchor ? anchor.before(button) : header.append(button);
  document.body.append(dialog);
}
