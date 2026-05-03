import { interpolate } from "remotion";
export function moveTextAnimation(frame: number, currentX: number) {
  const x = interpolate(frame, [0, 30], [currentX, currentX + 20]);
  return x;
}
