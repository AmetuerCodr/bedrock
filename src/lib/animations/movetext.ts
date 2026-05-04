import { spring } from "remotion";
export function moveTextAnimation(frame: number, fps: number, pixels: number) {
  const translation = spring({
    frame,
    fps,
    config: { damping: 12 },
  });
  const move = translation * pixels;
  return move;
}
