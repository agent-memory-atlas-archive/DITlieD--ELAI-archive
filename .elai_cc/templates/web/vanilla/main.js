const SECTION_FILES = [
  "sections/Hero.html",
  "sections/Features.html",
  "sections/Gallery.html",
  "sections/Cta.html",
  "sections/Footer.html",
];

async function loadSection(path) {
  const res = await fetch(path);
  if (!res.ok) {
    throw new Error("failed to load " + path + ": " + res.status);
  }
  return await res.text();
}

async function main() {
  const host = document.getElementById("app");
  if (!host) {
    throw new Error("mount target #app not found");
  }
  for (const file of SECTION_FILES) {
    const html = await loadSection(file);
    const wrap = document.createElement("div");
    wrap.innerHTML = html;
    host.appendChild(wrap);
  }
}

main().catch((err) => {
  console.error(err);
});
