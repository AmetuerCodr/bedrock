import { spring, interpolate } from "remotion";
export function letterDrift(
  frame: number,
  fps: number,
  letterIndex: number,
  delay = 3,
) {
  const seed = letterIndex * 7.3;
  const ox = Math.sin(seed) * 60;
  const oy = Math.cos(seed * 1.3) * 50;
  const rot = Math.sin(seed * 2.1) * 25;

  const progress = spring({
    frame: frame - letterIndex * delay,
    fps,
    config: { damping: 14, stiffness: 120, mass: 0.8 },
  });

  return {
    x: interpolate(progress, [0, 1], [ox, 0]),
    y: interpolate(progress, [0, 1], [oy, 0]),
    rotate: interpolate(progress, [0, 1], [rot, 0]),
    opacity: interpolate(progress, [0, 0.3], [0, 1], {
      extrapolateRight: "clamp",
    }),
  };
}
