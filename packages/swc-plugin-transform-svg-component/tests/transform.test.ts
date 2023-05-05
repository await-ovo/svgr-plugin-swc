/**
 * To ensure the same functionality as babel-plugin-transform-svg-component, these test cases modified from:
 * https://github.com/gregberge/svgr/blob/main/packages/babel-plugin-transform-svg-component/src/index.test.ts
 */
import { transformSync, parseSync } from '@swc/core';

const defaultOptions = {
  namedExport: 'ReactComponent',
  state: {
    componentName: 'SvgComponent',
    caller: {
      previousExport: '',
    },
  },
  expandProps: false,
};

const testPlugin =
  (language: string) =>
  (code: string, options: any = {}) => {
    const ast = parseSync(code, {
      syntax: 'ecmascript',
      jsx: true,
    });
    const result = transformSync(ast, {
      jsc: {
        parser: {
          syntax: 'ecmascript',
          jsx: false,
        },
        preserveAllComments: true,
        target: 'esnext',
        experimental: {
          plugins: [
            [
              require.resolve(
                '../../../target/wasm32-wasi/release/swc_plugin_transform_svg_component.wasm',
                {
                  paths: [__dirname],
                },
              ),
              {
                typescript: language === 'typescript',
                ...defaultOptions,
                ...options,
              },
            ],
          ],
        },
      },
    });

    if (!result) {
      throw new Error(`No result`);
    }

    return result;
  };

describe('plugin', () => {
  describe.each(['javascript', 'typescript'])('%s', (language) => {
    it('transforms whole program', () => {
      const { code } = testPlugin(language)('<svg><g /></svg>', {
      });
      expect(code).toMatchSnapshot();
    });

      describe('with "native" option', () => {
        it('adds import from "react-native-svg"', () => {
          // TODO: test it with  <Svg><g /></Svg>
          const { code } = testPlugin(language)('<svg><g /></svg>', {
            native: true,
          });
          expect(code).toMatchSnapshot();
        });
      });

      describe('with "ref" option', () => {
        it('adds ForwardRef component', () => {
          const { code } = testPlugin(language)('<svg><g /></svg>', {
            ref: true,
          });
          expect(code).toMatchSnapshot();
        });
      });

      describe('with "titleProp"', () => {
        it('adds "title" and "titleId" prop', () => {
          const { code } = testPlugin(language)('<svg><g /></svg>', {
            titleProp: true,
          });
          expect(code).toMatchSnapshot();
        });
      });

      describe('with "titleProp" and "expandProps"', () => {
        it('adds "title", "titleId" props and expands props', () => {
          const { code } = testPlugin(language)('<svg><g /></svg>', {
            ...defaultOptions,
            expandProps: true,
            titleProp: true,
          });
          expect(code).toMatchSnapshot();
        });
      });

      describe('with "descProp"', () => {
        it('adds "desc" and "descId" prop', () => {
          const { code } = testPlugin(language)('<svg><g /></svg>', {
            descProp: true,
          });
          expect(code).toMatchSnapshot();
        });
      });

      describe('with "descProp" and "expandProps"', () => {
        it('adds "desc", "descId" props and expands props', () => {
          const { code } = testPlugin(language)('<svg><g /></svg>', {
            ...defaultOptions,
            expandProps: true,
            descProp: true,
          });
          expect(code).toMatchSnapshot();
        });
      });

      describe('with "titleProp" and "descProp"', () => {
        it('adds "title", "titleId", "desc", and "descId prop', () => {
          const { code } = testPlugin(language)('<svg><g /></svg>', {
            titleProp: true,
            descProp: true,
          });
          expect(code).toMatchSnapshot();
        });
      });

      describe('with "titleProp" "descProp" and "expandProps"', () => {
        it('adds "title", "titleId", "desc", "descId" props and expands props', () => {
          const { code } = testPlugin(language)('<svg><g /></svg>', {
            ...defaultOptions,
            expandProps: true,
            titleProp: true,
            descProp: true,
          });
          expect(code).toMatchSnapshot();
        });
      });

      describe('with "expandProps"', () => {
        it('add props', () => {
          const { code } = testPlugin(language)('<svg><g /></svg>', {
            ...defaultOptions,
            state: { componentName: 'SvgComponent' },
            expandProps: true,
          });
          expect(code).toMatchSnapshot();
        });
      });

      describe('with "ref" and "expandProps" option', () => {
        it('expands props', () => {
          const { code } = testPlugin(language)('<svg><g /></svg>', {
            state: { componentName: 'SvgComponent' },
            expandProps: true,
            ref: true,
          });
          expect(code).toMatchSnapshot();
        });
      });

      describe('with "native", "ref" option', () => {
        it('adds import from "react-native-svg" and adds ForwardRef component', () => {
          // TODO: <Svg><g /></Svg>
          const { code } = testPlugin(language)('<svg><g /></svg>', {
            state: { componentName: 'SvgComponent' },
            native: true,
            ref: true,
          });
          expect(code).toMatchSnapshot();
        });
      });

      describe('with "native" and "expandProps" option', () => {
        it('adds import from "react-native-svg" and adds props', () => {
          // TODO: <Svg><g /></Svg>
          const { code } = testPlugin(language)('<svg><g /></svg>', {
            state: { componentName: 'SvgComponent' },
            native: true,
            expandProps: true,
          });
          expect(code).toMatchSnapshot();
        });
      });

      describe('with "native", "ref" and "expandProps" option', () => {
        it('adds import from "react-native-svg" and adds props and adds ForwardRef component', () => {
          // TODO: <Svg><g /></Svg>
          const { code } = testPlugin(language)('<svg><g /></svg>', {
            state: { componentName: 'SvgComponent' },
            native: true,
            expandProps: true,
            ref: true,
          });
          expect(code).toMatchSnapshot();
        });
      });

      describe('with "memo" option', () => {
        it('wrap component in "React.memo"', () => {
          const { code } = testPlugin(language)('<svg><g /></svg>', {
            state: { componentName: 'SvgComponent' },
            memo: true,
          });
          expect(code).toMatchSnapshot();
        });
      });

      describe('with both "memo" and "ref" option', () => {
        it('wrap component in "React.memo" and "React.forwardRef"', () => {
          const { code } = testPlugin(language)('<svg><g /></svg>', {
            state: { componentName: 'SvgComponent' },
            memo: true,
            ref: true,
          });
          expect(code).toMatchSnapshot();
        });
      });

      describe('with "namedExport" option and "previousExport" state', () => {
        it('has custom named export', () => {
          const { code } = testPlugin(language)('<svg><g /></svg>', {
            state: {
              componentName: 'SvgComponent',
              caller: {
                previousExport: `var img = new Image(); img.src = '...'; export default img;`,
              },
            },
            namedExport: 'Component',
          });
          expect(code).toMatchSnapshot();
        });
      });

      describe('with "namedExport" and "exportType" option and without "previousExport" state', () => {
        it('exports via named export', () => {
          const { code } = testPlugin(language)('<svg><g /></svg>', {
            state: {
              componentName: 'SvgComponent',
              caller: { previousExport: "" },
            },
            namedExport: 'ReactComponent',
            exportType: 'named',
          });
          expect(code).toMatchSnapshot();
        });
      });

      describe('#jsxRuntime', () => {
        it('supports "automatic" jsxRuntime', () => {
          const { code } = testPlugin(language)('<svg><g /></svg>', {
            jsxRuntime: 'automatic',
          });
          expect(code).toMatchSnapshot();
        });

        it('supports "classic" jsxRuntime', () => {
          const { code } = testPlugin(language)('<svg><g /></svg>', {
            jsxRuntime: 'classic',
          });
          expect(code).toMatchSnapshot();
        });

        it('allows to specify a custom "classic" jsxRuntime using "specifiers"', () => {
          const { code } = testPlugin(language)('<svg><g /></svg>', {
            jsxRuntime: 'classic',
            jsxRuntimeImport: { specifiers: ['h'], source: 'preact' },
          });
          expect(code).toMatchSnapshot();
        });

        it('allows to specify a custom "classic" jsxRuntime using "namespace"', () => {
          const { code } = testPlugin(language)('<svg><g /></svg>', {
            jsxRuntime: 'classic',
            jsxRuntimeImport: { namespace: 'Preact', source: 'preact' },
          });
          expect(code).toMatchSnapshot();
        });

        it('allows to specify a custom "classic" jsxRuntime using "defaultSpecifier"', () => {
          const { code } = testPlugin(language)('<svg><g /></svg>', {
            jsxRuntime: 'classic',
            jsxRuntimeImport: {
              defaultSpecifier: 'h',
              source: 'hyperapp-jsx-pragma',
            },
          });
          expect(code).toMatchSnapshot();
        });

        // TODO: panic message
        // it('throws with invalid configuration', () => {
        //   expect(() => {
        //     testPlugin(language)('<svg><g /></svg>', {
        //       jsxRuntime: 'classic',
        //       jsxRuntimeImport: { source: 'preact' },
        //     });

        //   }).toThrow();
        // });
      });

      it('allows to specify a different import source', () => {
        const { code } = testPlugin(language)('<svg><g /></svg>', {
          memo: true,
          ref: true,
          importSource: 'preact/compat',
          jsxRuntimeImport: { specifiers: ['h'], source: 'preact' },
        });
        expect(code).toMatchSnapshot();
      });
  });
});
