const htmlTemplate = await Bun.file("./index.html").text();
const bundleJs = await Bun.file("./dist/bundle.js").text();
const themeCss = await Bun.file("./src/styles/theme.css").text();

let inlined = htmlTemplate
  .replace('<link rel="stylesheet" href="./src/styles/theme.css">', `<style>\n${themeCss}\n</style>`)
  .replace('<script type="module" src="./src/main.ts"></script>', `<script>\n${bundleJs}\n</script>`);

await Bun.write("./dist/index.html", inlined);
console.log("Built standalone production bundle at ui/dist/index.html (" + inlined.length + " bytes)");

export {};
