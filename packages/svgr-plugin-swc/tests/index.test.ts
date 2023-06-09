/**
 * Modified from https://github.com/gregberge/svgr/blob/main/packages/plugin-jsx/src/index.test.ts
 */
import jsx from '../src/index';

const svgBaseCode = `
<svg width="88px" height="88px" viewBox="0 0 88 88" version="1.1" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink">
    <title>Dismiss</title>
    <desc>Created with Sketch.</desc>
    <defs></defs>
    <g id="Blocks" stroke="none" stroke-width="1" fill="none" fill-rule="evenodd" stroke-linecap="square">
        <g id="Dismiss" stroke="#063855" stroke-width="2">
            <path d="M51,37 L37,51" id="Shape"></path>
            <path d="M51,51 L37,37" id="Shape"></path>
        </g>
    </g>
</svg>
`;

describe('plugin', () => {
  it('transforms code', () => {
    const result = jsx(svgBaseCode, {}, { componentName: 'SvgComponent' });
    expect(result).toMatchSnapshot();
  });

  it('supports "automatic" runtime', () => {
    const result = jsx(
      svgBaseCode,
      { jsxRuntime: 'automatic' },
      { componentName: 'SvgComponent' },
    );
    expect(result).toMatchSnapshot();
  });

  it('supports "preact" preset', () => {
    const result = jsx(
      svgBaseCode,
      { jsxRuntime: 'classic-preact' },
      { componentName: 'SvgComponent' },
    );
    expect(result).toMatchSnapshot();
  });
});
