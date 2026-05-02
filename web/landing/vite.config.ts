import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { resolve } from "path";

export default defineConfig({
  plugins: [react()],
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
