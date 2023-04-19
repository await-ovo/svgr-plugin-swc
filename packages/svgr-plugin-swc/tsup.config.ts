import { defineConfig } from 'tsup';

export default defineConfig({
  entry: ['./src/index.ts'],
  format: ["cjs"],
  dts: true,
  clean: true,
  external: [
    "swc-plugin-add-jsx-attribute",
    "swc-plugin-add-jsx-attribute",
    "swc-plugin-remove-jsx-attribute",
    "swc-plugin-remove-jsx-empty-expression",
    "swc-plugin-replace-jsx-attribute-value",
    "swc-plugin-svg-dynamic-title",
    "swc-plugin-svg-em-dimensions",
    "swc-plugin-transform-react-native-svg",
    "swc-plugin-transform-svg-component",
  ],
});
