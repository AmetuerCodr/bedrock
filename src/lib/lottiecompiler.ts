import { resolve } from "node:path";
import { VisualMomentArraySchema, type VisualMoment } from "./schema";
import { appendFile } from "node:fs/promises";

export async function runLottieCompiler(
  videoData: unknown,
): Promise<VisualMoment[]> {
  const binPath = resolve(import.meta.dir, "compiler");

  const proc = Bun.spawn([binPath], {
    stdin: "pipe",
    stdout: "pipe",
    stderr: "inherit",
  });

  proc.stdin.write(JSON.stringify(videoData));
  proc.stdin.end();

  const [stdoutText, exitCode] = await Promise.all([
    new Response(proc.stdout).text(),
    proc.exited,
  ]);

  // TODO: if the error code is 503, switch models

  if (exitCode !== 0) {
    throw new Error(`compiler exited with code ${exitCode}`);
  }

  const parsed = JSON.parse(stdoutText.trim());
  return VisualMomentArraySchema.parse(parsed);
}

if (import.meta.main) {
  const data = await Bun.file("./public/data.json").json();
  const moments = await runLottieCompiler(data);
  console.log(JSON.stringify(moments, null, 2));
  await Bun.write(
    "./public/lottiemoments.json",
    JSON.stringify(moments, null, 2),
  );
}
