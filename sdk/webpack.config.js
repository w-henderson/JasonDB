import path from "path";

export default {
  entry: "./lib/index.js",
  mode: "production",
  output: {
    path: path.resolve("dist"),
    filename: "jasondb.js",
    library: "JasonDB",
    libraryTarget: "umd",
    libraryExport: "default"
  },
};