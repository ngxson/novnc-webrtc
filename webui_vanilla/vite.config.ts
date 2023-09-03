import { defineConfig, PluginOption } from "vite"
import { viteSingleFile } from "vite-plugin-singlefile"
import fs from "fs"
import path from "path"

const NOVNC_BASE_PATH = path.join(__dirname, './src/noVNC');

// polyfill
const replaceAll = function (str: string, search: string, replacement: string) {
  return str.replace(new RegExp(search, 'g'), replacement);
};
const base64 = (str: string) => Buffer.from(str).toString('base64');

// simple plugin that makes svg inline inside css file
function svgInlineCss() {
  return {
    name: 'svg-inline-css-loader',
    load(id: string) {
      if (id.endsWith('.css')) {
        let code = fs.readFileSync(id).toString();
        const svgPaths = code.match(/[\_\-0-9a-zA-Z\/\\\.]+\.svg/g);
        if (!svgPaths) return; // do nothing
        for (let i = 0; i < svgPaths.length; i++) {
          const svgPath = path.join(id, '..', svgPaths[i]);
          const svgData = fs.readFileSync(svgPath).toString();
          const inlinedSvg = `data:image/svg+xml;base64,${base64(svgData)}`;
          // console.log(`Inlining svg: ${svgPath}`);
          code = replaceAll(code, svgPaths[i], inlinedSvg);
        }
        return code;
      }
    },
  } as PluginOption;
}

// simple plugin that inline all assets inside index.html
function assetsInlineIndexHtml() {
  return {
    name: 'assets-inline-index-html-loader',
    transformIndexHtml: {
      enforce: 'pre',
      transform(html: string) {
        const assets = html.match(/app\/(images|sounds)\/[^"']+/g);
        if (!assets) return html; // do nothing
        let output = html;
        for (let i = 0; i < assets.length; i++) {
          const assetPath = path.join(NOVNC_BASE_PATH, assets[i]);
          const assetData = fs.readFileSync(assetPath).toString();
          let mime = '';
          if (assetPath.endsWith('.svg')) {
            mime = 'data:image/svg+xml';
          } else if (assetPath.endsWith('.png')) {
            mime = 'data:image/png';
          } else if (assetPath.endsWith('.ico')) {
            mime = 'data:image/x-icon';
          } else if (assetPath.endsWith('.mp3')) {
            mime = 'data:audio/mp3';
          } else if (assetPath.endsWith('.oga')) {
            mime = 'data:audio/ogg';
          } else {
            throw Error(`Undefined MIME for ${assetPath}`);
          }
          const inlined = `${mime};base64,${base64(assetData)}`;
          output = replaceAll(output, assets[i], inlined);
        }
        return output;
      }
    },
  } as PluginOption;
}

export default defineConfig({
  plugins: [assetsInlineIndexHtml(), svgInlineCss(), viteSingleFile()],
})