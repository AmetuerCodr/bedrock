// render.ts  (at project root, next to package.json)
import { bundle } from "@remotion/bundler";
import { renderMedia, selectComposition } from "@remotion/renderer";

async function readFileData() {
  const file = Bun.file("./public/data.json");
  const contents = await file.text();

  const cleanedContents = contents
    .replace(/^```json\s*/i, "")
    .replace(/```\s*$/i, "")
    .trim();
  const json = JSON.parse(cleanedContents);
  console.log("json:", json);
  return json;
}
interface DataFileContent {
  script: string;
  wordGroups: string[];
  isDisplayFont: boolean[];
}

async function main() {
  const content = await readFileData();
  console.log(typeof content);
  const data: DataFileContent = content;
  const bundled = await bundle({ entryPoint: "./src/index.ts" });

  const composition = await selectComposition({
    serveUrl: bundled,
    id: "KineticTypography", // must match what's in Root.tsx
    inputProps: { script: data.wordGroups, displayFontArray: data.isDisplayFont },
  });

  await renderMedia({
    composition,
    serveUrl: bundled,
    codec: "h264",
    outputLocation: "out/video.mp4",
    inputProps: { script: data.wordGroups },
  });

  console.log("🎬 Rendered to out/video.mp4");
}

main();
