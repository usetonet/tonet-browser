import { defineConfig } from "vite";
import { resolve } from "path";

export default defineConfig({
  root: ".",
  appType: "mpa",
  build: {
    outDir: "dist",
    emptyOutDir: true,
    rollupOptions: {
      input: {
        main: resolve(__dirname, "index.html"),
        docs: resolve(__dirname, "docs.html"),
        compare: resolve(__dirname, "compare.html"),
        roadmap: resolve(__dirname, "roadmap.html"),
        handbook: resolve(__dirname, "handbook.html"),
      },
    },
  },
});
