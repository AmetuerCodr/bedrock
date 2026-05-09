async function compiler() {
  const proc = Bun.spawn(["src/lib/compiler"], {
    stdin: "pipe",
  });

  // proc.stdin.write("Shammah Womack");
  // proc.stdin.end(); // Close stdin to allow the process to finish
  const output = await new Response(proc.stdout).text();
  console.log(output);
}

compiler();
