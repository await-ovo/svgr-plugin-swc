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
      require.resolve('swc-plugin-svgr'),
      {
        transform_svg_component: {
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
        em_dimensions:
          config.icon !== false && config.dimensions
            ? config.icon !== true
              ? { width: config.icon, height: config.icon }
              : config.native
              ? {
                  width: 24,
                  height: 24,
                }
              : {}
            : undefined,
        remove_jsx_attribute:  {
          elements: ['svg', 'Svg'],
          attributes: toRemoveAttributes,
        },
        add_jsx_attribute:   {
          elements: ['svg', 'Svg'],
          attributes: toAddAttributes,
        },
        replace_attribute_values: config.replaceAttrValues ? {
          values:  replaceMapToValues(config.replaceAttrValues) 
        } : undefined,
        title_prop: Boolean(config.titleProp),
        desc_prop: Boolean(config.descProp),
        native: Boolean(config.native),
      },
    ],
  ] as SwcPluginOptions;
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
