<h1 align='center'>svgr-plugin-swc</h1>

<p align='center'>
<em>The <a href="https://github.com/gregberge/svgr" target="_blank">SVGR</a> plugin for transformation using <a href="https://github.com/swc-project/swc" target="_blank">SWC</a></em>
<br />
<br />
<a href='https://www.npmjs.com/package/svgr-plugin-swc'>
<img src='https://img.shields.io/npm/v/svgr-plugin-swc/latest.svg'>
</a>
<a href='https://npmjs.com/package/svgr-plugin-swc'>
<img src='https://img.shields.io/npm/l/svgr-plugin-swc' >
</a>
</p>


A plugin for [SVGR](https://github.com/gregberge/svgr) that replaces the `@svgr/plugin-jsx` plugin. This plugin uses the [SWC](https://github.com/swc-project/swc) compiler under the hood to provide a faster JSX transformation.

## Installation

```bash
pnpm add svgr-plugin-swc -D
```

## Usage

Add the plugin to your SVGR configuration:

```js
// .svgrc.js
module.exports = {
  plugins: [
    '@svgr/plugin-svgo',
    'svgr-plugin-swc',
  ],
  expandProps: false
};
```

Note that when using `svgr-plugin-swc`, we must also use `@svgr/plugin-svgo` to remove the comments and XML processing instructions in svg file.

This plugin supports all the options that the `@svgr/plugin-jsx` plugin supports, **except for the `template` and `jsx.babelConfig` option**.

### Example

```
//example.svg
<?xml version="1.0" encoding="UTF-8"?>
<svg width="48px" height="1px" viewBox="0 0 48 1" version="1.1" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink">
    <!-- Generator: Sketch 46.2 (44496) - http://www.bohemiancoding.com/sketch -->
    <title>Rectangle 5</title>
    <defs></defs>
    <g id="Page-1" stroke="none" stroke-width="1" fill="none" fill-rule="evenodd">
        <g id="19-Separator" transform="translate(-129.000000, -156.000000)" fill="#063855">
            <g id="Controls/Settings" transform="translate(80.000000, 0.000000)">
                <g id="Content" transform="translate(0.000000, 64.000000)">
                    <g id="Group" transform="translate(24.000000, 56.000000)">
                        <g id="Group-2">
                            <rect id="Rectangle-5" x="25" y="36" width="48" height="1"></rect>
                        </g>
                    </g>
                </g>
            </g>
        </g>
    </g>
</svg>

// index.mjs
import { transform } from "@svgr/core";
import fs from "fs";
const svgCode = fs.readFileSync("./example.svg");
transform(
  svgCode,
  {
    native: true,
    plugins: [
      "@svgr/plugin-svgo", 
      "svgr-plugin-swc",
      "@svgr/plugin-prettier"
    ],
    typescript: false,
    descProp: true,
    titleProp: true,
    ref: true,
    dimensions: false,
    svgProps: { size: "1em" },
  },
  { componentName: "MyComponent" }
).then((jsCode) => {
  fs.writeFileSync("./output.jsx", jsCode, "utf8");
});


```
`output.jsx` should output as:

```
import * as React from "react";
import Svg, { Path } from "react-native-svg";
import { forwardRef } from "react";
const MyComponent = ({ title, titleId, desc, descId, ...props }, ref) => (
  <Svg
    xmlns="http://www.w3.org/2000/svg"
    size="1em"
    ref={ref}
    aria-labelledby={titleId}
    aria-describedby={descId}
    {...props}
  >
    {desc ? <desc id={descId}>{desc}</desc> : null}
    {title ? <title id={titleId}>{title}</title> : null}
    <Path fill="#063855" fillRule="evenodd" d="M0 0h48v1H0z" />
  </Svg>
);
const ForwardRef = forwardRef(MyComponent);
export default ForwardRef;
```


## License

MIT