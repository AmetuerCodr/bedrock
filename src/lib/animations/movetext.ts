import { interpolate, spring } from "remotion";
export function moveTextAnimation(
  frame: number,
  currentX: number,
  currentY: number,
) {
  const x = interpolate(frame, [0, 30], [currentX, currentX + 20]);
  const y = interpolate(frame, [0, 30], [currentY, currentY + 20]);
  return {
    x: x,
    y: y,
  };
}
