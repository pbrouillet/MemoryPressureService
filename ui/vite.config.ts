import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { viteSingleFile } from "vite-plugin-singlefile";
import path from "path";

export default defineConfig(({ mode }) => {
  const isSettings = mode === "settings";
  const entry = isSettings ? "settings.html" : "stats.html";

  return {
    plugins: [react(), viteSingleFile()],
    root: ".",
    build: {
      outDir: "dist",
      emptyOutDir: false,
      rollupOptions: {
        input: entry,
      },
    },
    resolve: {
      alias: {
        "@shared": path.resolve(__dirname, "src/shared"),
      },
    },
  };
});
