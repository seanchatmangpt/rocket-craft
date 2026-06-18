import js from "@eslint/js";
import eslintConfigPrettier from "eslint-config-prettier";
import eslintPluginPrettierRecommended from "eslint-plugin-prettier/recommended";

export default [
  js.configs.recommended,
  eslintPluginPrettierRecommended,
  {
    languageOptions: {
      ecmaVersion: "latest",
      sourceType: "module",
      globals: {
        document: "readonly",
        window: "readonly",
        console: "readonly",
        fetch: "readonly",
        navigator: "readonly",
        URL: "readonly",
        setTimeout: "readonly",
        clearTimeout: "readonly",
        setInterval: "readonly",
        clearInterval: "readonly",
        btoa: "readonly",
        atob: "readonly",
        sessionStorage: "readonly",
        localStorage: "readonly",
        performance: "readonly",
        self: "readonly",
        caches: "readonly",
        clients: "readonly",
        Headers: "readonly",
        Request: "readonly",
        Response: "readonly",
        Promise: "readonly"
      }
    },
    rules: {
      "no-console": "off",
      "no-unused-vars": ["warn", { "argsIgnorePattern": "^event" }]
    }
  },
  {
    ignores: [
      "dist/",
      "node_modules/",
      "*.data",
      "*.wasm",
      "*.js.symbols",
      "RealisticRendering-HTML5-Shipping.js",
      "Brm-HTML5-Shipping.js",
      "ShooterGame-HTML5-Shipping.js",
      "FullSpectrum-HTML5-Shipping.js",
      "coverage/"
    ]
  }
];
