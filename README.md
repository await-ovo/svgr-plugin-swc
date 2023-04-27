# svgr-plugin-swc(Under Development)

A plugin for [SVGR](https://github.com/gregberge/svgr) that replaces the `@svgr/plugin-jsx` plugin. This plugin uses the [swc](https://github.com/swc-project/swc) compiler under the hood to provide a faster JSX transformation.

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
    'svgr-plugin-svgo',
    'svgr-plugin-swc',
  ],
};
```

You can also pass options to the plugin:

```js
// .svgrc.js
module.exports = {
  plugins: [
    [
      'svgr-plugin-swc',
      {
        typescript: true,
        // other options
      },
    ],
    // other plugins
  ],
};
```

This plugin supports all the options that the `@svgr/plugin-jsx` plugin supports, **except for the `template` option**.

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
    icon: true,
    native: true,
    plugins: ["@svgr/plugin-svgo", "svgr-plugin-swc"],
    typescript: false,
    descProp: true,
    titleProp: true,
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
const MyComponent = ({ title, titleId, desc, descId, ...props }) => (
  <Svg
    xmlns="http://www.w3.org/2000/svg"
    width={48}
    height={1}
    size="1em"
    aria-labelledby={titleId}
    aria-describedby={descId}
    {...props}
  >
    {desc ? <desc id={descId}>{desc}</desc> : null}
    {title ? <title id={titleId}>{title}</title> : null}
    <Path fill="#063855" fillRule="evenodd" d="M0 0h48v1H0z" />
  </Svg>
);
export default MyComponent;

```


## License

MIT