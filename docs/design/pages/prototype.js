const lucide = (body) =>
  `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">${body}</svg>`;

const icons = {
  alert: lucide('<circle cx="12" cy="12" r="10"/><line x1="12" x2="12" y1="8" y2="12"/><line x1="12" x2="12.01" y1="16" y2="16"/>'),
  check: lucide('<circle cx="12" cy="12" r="10"/><path d="m9 12 2 2 4-4"/>'),
  loader: lucide('<path d="M21 12a9 9 0 1 1-6.219-8.56"/>'),
  google: document.querySelector("#icon-google")?.innerHTML ?? "",
};

for (const slot of document.querySelectorAll("[data-icon]")) {
  slot.innerHTML = icons[slot.dataset.icon];
}

const params = new URLSearchParams(location.search);
if (params.has("dark")) document.documentElement.classList.add("dark");
if (params.has("desktop")) document.body.dataset.mode = "desktop";
