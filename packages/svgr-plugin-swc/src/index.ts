/**
 * Modified from https://github.com/gregberge/svgr/blob/main/packages/babel-preset/src/index.ts
 */
import { parseSync, transformSync } from '@swc/core';
import type {
  Attribute,
  SwcPluginOptions,
  TransformSvgComponentOptions,
} from './types';
import type { Plugin, Config, State } from '@svgr/core';
import {
  getJsxRuntimeOptions,
  propsToAttributes,
  replaceMapToValues,
} from './util';

const getPlugins = (config: Config, state: State): SwcPluginOptions => {
  let toRemoveAttributes = ['version'];
  let toAddAttributes: Attribute[] = [];

  if (config.svgProps) {
    toAddAttributes = [
      ...toAddAttributes,
      ...propsToAttributes(config.svgProps),
    ];
  }

  if (config.ref) {
    toAddAttributes.push({
      name: 'ref',
      value: 'ref',
      literal: true,
    });
  }

  if (config.titleProp) {
    toAddAttributes.push({
      name: 'aria-labelledby',
      value: 'titleId',
      literal: true,
    });
  }

  if (config.descProp) {
    toAddAttributes.push({
      name: 'aria-describedby',
      value: 'descId',
      literal: true,
    });
  }

  if (config.expandProps) {
    toAddAttributes.push({
      name: 'props',
      spread: true,
      position:
        config.expandProps === 'start' || config.expandProps === 'end'
          ? config.expandProps
          : undefined,
    });
  }

  if (!config.dimensions) {
    toRemoveAttributes = [...toRemoveAttributes, 'width', 'height'];
  }

  const plugins = [
    [
      require.resolve('swc-plugin-transform-svg-component'),
      {
        typescript: config.typescript,
        titleProp: config.titleProp,
        descProp: config.descProp,
        expandProps: config.expandProps,
        ref: config.ref,
        state,
        native: config.native,
        memo: config.memo,
        exportType: config.exportType,
        namedExport: config.namedExport,
        ...getJsxRuntimeOptions(config),
      } as TransformSvgComponentOptions,
    ],
    config.icon !== false &&
      config.dimensions && [
        require.resolve('swc-plugin-svg-em-dimensions'),
        config.icon !== true
          ? { width: config.icon, height: config.icon }
          : config.native
          ? {
              width: 24,
              height: 24,
            }
          : {},
      ],
    [
      require.resolve('swc-plugin-remove-jsx-attribute'),
      {
        elements: ['svg', 'Svg'],
        attributes: toRemoveAttributes,
      },
    ],
    [
      require.resolve('swc-plugin-add-jsx-attribute'),
      {
        elements: ['svg', 'Svg'],
        attributes: toAddAttributes,
      },
    ],
    [require.resolve('swc-plugin-remove-jsx-empty-expression'), {}],
  ].filter(Boolean) as SwcPluginOptions;

  if (config.replaceAttrValues) {
    plugins.push([
      require.resolve('swc-plugin-replace-jsx-attribute-value'),
      {
        values: replaceMapToValues(config.replaceAttrValues),
      },
    ]);
  }

  if (config.titleProp) {
    plugins.push([require.resolve('swc-plugin-svg-dynamic-title'), {}]);
  }

  if (config.descProp) {
    plugins.push([require.resolve('swc-plugin-svg-dynamic-title'), { tag: 'desc' }])
  }

  if (config.native) {
    plugins.push([require.resolve('swc-plugin-transform-react-native-svg'), {}])
  }

  return plugins;
};

const swcPlugin: Plugin = (code, config, state) => {
  const filePath = state.filePath || 'unknown';

  const ast = parseSync(code, {
    syntax: 'ecmascript',
    jsx: true,
    comments: true,
  });

  const result = transformSync(ast, {
    filename: filePath,
    swcrc: false,
    configFile: false,
    caller: {
      name: 'svgr',
    },
    jsc: {
      parser: {
        syntax: 'ecmascript',
        jsx: false,
      },
      preserveAllComments: true,
      target: 'esnext',
      experimental: {
        plugins: getPlugins(config, state),
      },
    },
  });

  if (!result.code) {
    throw new Error(`Unable to generate SVG file`);
  }

  return result.code;
};

export default swcPlugin;
