import js from "@eslint/js";
import tseslint from "typescript-eslint";
import react from "eslint-plugin-react";
import reactHooks from "eslint-plugin-react-hooks";
import jsxA11y from "eslint-plugin-jsx-a11y";
import prettier from "eslint-config-prettier";
import globals from "globals";

const noRawHex = {
  rules: {
    "no-raw-hex": {
      meta: {
        type: "problem",
        docs: {
          description:
            "Disallow raw hex colors outside theme.css. All colors must reference DESIGN.md tokens via Tailwind utilities.",
        },
        schema: [],
        messages: {
          rawHex:
            "Raw hex color {{hex}} is forbidden outside theme.css. Reference a token via Tailwind utility (e.g., bg-primary, text-foreground).",
        },
      },
      create(context) {
        const HEX = /#[0-9a-fA-F]{3,8}\b/g;
        function checkString(node, value) {
          let match;
          while ((match = HEX.exec(value)) !== null) {
            context.report({ node, messageId: "rawHex", data: { hex: match[0] } });
          }
        }
        return {
          Literal(node) {
            if (typeof node.value === "string") checkString(node, node.value);
          },
          TemplateElement(node) {
            checkString(node, node.value.raw);
          },
          JSXAttribute(node) {
            if (
              node.value &&
              node.value.type === "Literal" &&
              typeof node.value.value === "string"
            ) {
              checkString(node, node.value.value);
            }
          },
        };
      },
    },
  },
};

export default tseslint.config(
  {
    ignores: [
      "dist",
      "src-tauri",
      "src/lib/bindings.ts",
      "src/styles/**",
      "node_modules",
      "coverage",
      "**/*.css",
    ],
  },
  js.configs.recommended,
  ...tseslint.configs.recommended,
  {
    files: ["**/*.{ts,tsx}"],
    languageOptions: {
      globals: { ...globals.browser, ...globals.es2024 },
      parserOptions: {
        ecmaVersion: 2024,
        sourceType: "module",
        ecmaFeatures: { jsx: true },
      },
    },
    plugins: {
      react,
      "react-hooks": reactHooks,
      "jsx-a11y": jsxA11y,
      ripley: noRawHex,
    },
    settings: { react: { version: "19" } },
    rules: {
      ...react.configs.recommended.rules,
      ...reactHooks.configs.recommended.rules,
      ...jsxA11y.configs.recommended.rules,
      "react/react-in-jsx-scope": "off",
      "react/prop-types": "off",
      "ripley/no-raw-hex": "error",
    },
  },
  {
    files: ["src/styles/**/*.css", "vite.config.ts", "vitest.config.ts"],
    rules: { "ripley/no-raw-hex": "off" },
  },
  prettier,
);
