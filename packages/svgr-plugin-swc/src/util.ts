import type { Config } from '@svgr/core';
import type { Attribute, TransformSvgComponentOptions, Value } from './types';

/**
 * To be consistent with @swr/plugin-jsx, the following function "getJsxRuntimeOptions" is modified from
 * https://github.com/gregberge/svgr/blob/main/packages/plugin-jsx/src/index.ts#L9
 */
export const getJsxRuntimeOptions = (
  config: Config,
): Partial<TransformSvgComponentOptions> => {
  if (config.jsxRuntimeImport) {
    return {
      importSource: config.jsxRuntimeImport.source,
      jsxRuntimeImport: config.jsxRuntimeImport,
    };
  }
  switch (config.jsxRuntime) {
    case null:
    case undefined:
    case 'classic':
      return {
        jsxRuntime: 'classic',
        importSource: 'react',
        jsxRuntimeImport: { namespace: 'React', source: 'react' },
      };
    case 'classic-preact':
      return {
        jsxRuntime: 'classic',
        importSource: 'preact/compat',
        jsxRuntimeImport: { specifiers: ['h'], source: 'preact' },
      };
    case 'automatic':
      return { jsxRuntime: 'automatic' };
    default:
      throw new Error(`Unsupported "jsxRuntime" "${config.jsxRuntime}"`);
  }
};

/**
 * To be consistent with @svgr/babel-preset the following functions("getAttributeValue", "propsToAttributes", "replaceMapToValues") is modified from
 * https://github.com/gregberge/svgr/blob/main/packages/babel-preset/src/index.ts#L30
 */
export const getAttributeValue = (value: string) => {
  const literal =
    typeof value === 'string' && value.startsWith('{') && value.endsWith('}');
  return { value: literal ? value.slice(1, -1) : value, literal };
};

export const propsToAttributes = (props: {
  [key: string]: string;
}): Attribute[] => {
  return Object.keys(props).map((name) => {
    const { literal, value } = getAttributeValue(props[name]);
    return { name, literal, value };
  });
};

export const replaceMapToValues = (replaceMap: {
  [key: string]: string;
}): Value[] => {
  return Object.keys(replaceMap).map((value) => {
    const { literal, value: newValue } = getAttributeValue(replaceMap[value]);
    return { value, newValue, literal };
  });
};
