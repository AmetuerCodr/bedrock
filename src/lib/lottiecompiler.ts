async function compiler() {
  const proc = Bun.spawn(["src/lib/compiler"]);

  await proc.exited;

  const usage = proc.resourceUsage();
  console.log(`Max memory used: ${usage?.maxRSS} bytes`);
  console.log(`CPU time (user): ${usage?.cpuTime.user} µs`);
  console.log(`CPU time (system): ${usage?.cpuTime.system} µs`);
}

compiler();
