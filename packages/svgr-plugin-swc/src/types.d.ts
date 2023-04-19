export type Attribute =  {
  name: string;
  value?: boolean | number | string | null;
  spread?: boolean;
  literal?: boolean;
  position?: 'start' | 'end';
}

export type Value = {
  value: string;
  newValue: string | boolean | number;
  literal?: boolean;
};

export type SwcPluginOptions = Array<[string, Record<string, any>]>;

export type TransformSvgComponentOptions = {
  typescript?: boolean;
  titleProp?: boolean;
  descProp?: boolean;
  expandProps?: boolean | 'start' | 'end';
  ref?: boolean;
  // The template option is not supported in swc-plugin-transform-svg-component.
  // template?: Template; 
  state: State;
  native?: boolean;
  memo?: boolean;
  exportType?: 'named' | 'default';
  namedExport?: string;
  jsxRuntime?: 'automatic' | 'classic';
  jsxRuntimeImport?: JSXRuntimeImport;
  importSource?: string;
};

