import { spring } from "remotion";
export function moveTextAnimation(
  frame: number,
  fps: number,
  pixels: number,
  delay: number = 0,
) {
  const translation = spring({
    frame: frame - delay,
    fps,
    config: { damping: 12 },
  });
  const move = delay ? Math.min(translation, 1) * pixels : 0;
  return { move };
}
