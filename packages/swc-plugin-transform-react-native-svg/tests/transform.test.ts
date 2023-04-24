import { transformSync, parseSync } from "@swc/core";

const testPlugin = (code: string) => {
  const ast = parseSync(code, {
    syntax: "ecmascript",
    jsx: true,
  });
  const result = transformSync(ast, {
    jsc: {
      parser: {
        syntax: "ecmascript",
        jsx: false,
      },
      preserveAllComments: true,
      target: "esnext",
      experimental: {
        plugins: [
          [
            require.resolve(
              "../../../target/wasm32-wasi/release/swc_plugin_transform_react_native_svg.wasm",
              {
                paths: [__dirname],
              }
            ),
            {},
          ],
        ],
      },
    },
  });
  return result?.code;
};

describe("plugin", () => {
  it("should transform elements", () => {
    const code = testPlugin("<svg><div /></svg>");
    expect(code).toBe("<Svg></Svg>;\n");
  });

  it("should add import", () => {
    const code = testPlugin(
      `import Svg, { Defs } from 'react-native-svg'; <svg><g /><div /><defs></defs></svg>;`
    );
    expect(code).toBe(
      "import Svg, { Defs, G } from 'react-native-svg';\n/* SVGR has dropped some elements not supported by react-native-svg: div */ <Svg><G/><Defs></Defs></Svg>;\n"
    );
  });
});
