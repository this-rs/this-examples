#!/usr/bin/env node
// Vérifie ce que j'ai mesuré à la main six fois en écrivant l'interface.
//
// Usage : node tools/check-ui.mjs [url]
//   par défaut http://127.0.0.1:8790/ — le serveur de démo, ou n'importe quel
//   serveur statique qui sert ui/dist.
//
// Ce qui est vérifié, et pourquoi :
//   - la géométrie des trois points suit la plateforme (macOS à gauche, Windows
//     à droite) : c'est ce qui a coûté un aller-retour, les points atterrissant
//     au milieu de la fenêtre ;
//   - le premier clic qui active la fenêtre n'agit pas, sauf sur les points :
//     le mesurer évite de croire qu'un clic sur une ligne de tableau ne fait
//     que ramener la fenêtre au premier plan ;
//   - les icônes sont des masques, pas des glyphes de police : un glyphe à 14px
//     est flou et sa forme change avec la police.

import { createRequire } from "node:module";
const require = createRequire(import.meta.url);
const puppeteer = require("puppeteer-core");

const url = process.argv[2] ?? "http://127.0.0.1:8790/";
const candidates = [
  process.env.CHROME_PATH,
  "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
  "/usr/bin/google-chrome",
  "/usr/bin/google-chrome-stable",
  "/usr/bin/chromium-browser",
  "/usr/bin/chromium",
].filter(Boolean);

const { existsSync } = await import("node:fs");
const executablePath = candidates.find((path) => existsSync(path));
if (!executablePath) {
  console.error(
    `aucun navigateur trouvé ; candidats : ${candidates.join(", ")}\n` +
      "définir CHROME_PATH",
  );
  process.exit(2);
}
console.log(`navigateur : ${executablePath}`);

const failures = [];
const check = (name, ok, detail) => {
  if (!ok) failures.push(`${name} — ${detail}`);
  console.log(`${ok ? "  ok " : "  KO "} ${name}${ok ? "" : ` — ${detail}`}`);
};

const browser = await puppeteer.launch({
  executablePath,
  headless: "new",
  args: ["--no-sandbox"],
});
const page = await browser.newPage();
await page.setViewport({ width: 1180, height: 800 });
await page.goto(url, { waitUntil: "domcontentloaded" });
await new Promise((resolve) => setTimeout(resolve, 2500));

// La coquille native montre les contrôles ; en navigateur, on force le style.
await page.addStyleTag({ content: ".win-controls{display:flex !important}" });

const geometry = async (platform) => {
  await page.evaluate((value) => {
    const root = document.documentElement;
    root.setAttribute("data-platform", value);
    root.setAttribute("data-titlebar", "overlay");
    root.setAttribute("data-fullscreen", "false");
  }, platform);
  await new Promise((resolve) => setTimeout(resolve, 150));
  return page.evaluate(() => {
    const box = document.querySelector(".win-controls").getBoundingClientRect();
    const close = document.querySelector(".win-btn.close").getBoundingClientRect();
    const zoom = document.querySelector(".win-btn.zoom").getBoundingClientRect();
    return {
      left: Math.round(box.x),
      right: Math.round(box.right),
      top: Math.round(box.y),
      close: Math.round(close.x),
      zoom: Math.round(zoom.x),
      viewport: window.innerWidth,
    };
  });
};

const macos = await geometry("macos");
check("macOS : points au coin gauche", macos.left === 3 && macos.top === 3, JSON.stringify(macos));
check("macOS : fermer à gauche, agrandir à droite", macos.close < macos.zoom, JSON.stringify(macos));

const windows = await geometry("windows");
check(
  "Windows : points au coin droit",
  windows.right === windows.viewport - 3,
  JSON.stringify(windows),
);
check("Windows : ordre inversé", windows.close > windows.zoom, JSON.stringify(windows));

// Icônes : un masque est appliqué, pas un glyphe de texte.
const icons = await page.evaluate(() =>
  [".close", ".minimize", ".zoom"].map((selector) => {
    const before = getComputedStyle(
      document.querySelector(`.win-btn${selector}`),
      "::before",
    );
    const mask = before.maskImage || before.webkitMaskImage || "none";
    return mask.includes("svg") || mask.includes("png");
  }),
);
check("icônes : masque sur les trois", icons.every(Boolean), JSON.stringify(icons));

// Premier clic : avalé hors des points, transmis dessus.
const clickable = () =>
  page.evaluate(() => {
    window.__hits = { nav: 0, btn: 0 };
    document
      .querySelector(".crumbs")
      .addEventListener("click", () => (window.__hits.nav += 1));
    document
      .querySelector(".win-btn.close")
      .addEventListener("click", () => (window.__hits.btn += 1));
  });

await clickable();
await page.evaluate(() => window.dispatchEvent(new Event("focus")));
await page.evaluate(() => document.querySelector(".crumbs").click());
const afterNav = await page.evaluate(() => window.__hits.nav);
check("premier clic hors des points : avalé", afterNav === 0, `nav=${afterNav}`);

await page.evaluate(() => window.dispatchEvent(new Event("focus")));
await page.evaluate(() => document.querySelector(".win-btn.close").click());
const afterBtn = await page.evaluate(() => window.__hits.btn);
check("premier clic sur un point : transmis", afterBtn === 1, `btn=${afterBtn}`);

await browser.close();

if (failures.length > 0) {
  console.error(`\n${failures.length} vérification(s) en échec`);
  process.exit(1);
}
console.log("\ntout est conforme");
