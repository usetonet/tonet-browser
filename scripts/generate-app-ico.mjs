#!/usr/bin/env node
/**
 * Regenerate crates/tonet/windows/app.ico from web/landing/public/tonet.svg
 * Requires: npx @resvg/resvg-js-cli, npx png-to-ico
 */
import { execFileSync } from "node:child_process";
import { createWriteStream } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(fileURLToPath(new URL(".", import.meta.url)), "..");
const svg = join(root, "web/landing/public/tonet.svg");
const out = join(root, "crates/tonet/windows/app.ico");
const png256 = join(tmpdir(), "tonet-ico-256.png");

execFileSync(
  "npx",
  ["--yes", "@resvg/resvg-js-cli", "--fit-width", "256", svg, png256],
  { stdio: "inherit", shell: true, cwd: root },
);

const ico = execFileSync("npx", ["--yes", "png-to-ico", png256], {
  encoding: "buffer",
  shell: true,
  cwd: root,
});

createWriteStream(out).end(ico);
console.log(`Wrote ${out} (${ico.length} bytes)`);
