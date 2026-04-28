import { interpolate, Easing } from "remotion";

/**
 * CLIP SWEEP REVEAL
 * A clip-path wipes left to right, revealing full-brightness text
 * from a dim ghost underneath. Staggered per line index.
 *
 * This one uses interpolate() directly (not spring) because clip-path
 * sweeps feel better with a custom cubic bezier — spring physics here
 * produce too much wobble on a hard geometric edge.
 *
 * Usage:
 *   // You need TWO layers per line: a dim "ghost" and a bright "revealed" layer.
 *   lines.map((text, i) => {
 *     const { clipPath, ghostOpacity } = clipSweep(frame, fps, i);
 *     return (
 *       <div style={{ position: 'relative' }}>
 *         // Ghost layer — always visible, dim
 *         <span style={{ opacity: ghostOpacity, color: '#333' }}>{text}</span>
 *         // Revealed layer — clipped until sweep passes
 *         <span style={{
 *           position: 'absolute', inset: 0,
 *           clipPath,
 *           color: '#f5f5f4',
 *         }}>
 *           {text}
 *         </span>
 *       </div>
 *     )
 *   })
 */
export function clipSweep(
  frame: number,
  fps: number,
  lineIndex: number,
  options: {
    delayPerLine?: number; // frames between each line's sweep
    durationFrames?: number; // how many frames the sweep takes
    ghostOpacity?: number; // opacity of the dim ghost text (0–1)
  } = {},
) {
  const {
    delayPerLine = 12,
    durationFrames = 18,
    ghostOpacity = 0.15,
  } = options;

  const localFrame = frame - lineIndex * delayPerLine;

  // Percent of sweep completed: 0% (fully hidden) → 100% (fully revealed)
  // Using a fast-in, eased-out cubic bezier for a mechanical sweep feel
  const sweepPercent = interpolate(localFrame, [0, durationFrames], [0, 100], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
    easing: Easing.bezier(0.76, 0, 0.24, 1),
  });

  // clip-path inset: inset(top right bottom left)
  // We animate the right inset from 100% (fully clipped) → 0% (fully revealed)
  const rightInset = 100 - sweepPercent;
  const clipPath = `inset(0 ${rightInset}% 0 0)`;

  // Ghost fades in slightly before the sweep starts, so there's something to reveal from
  const ghostFade = interpolate(localFrame, [-4, 2], [0, ghostOpacity], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  return {
    clipPath,
    ghostOpacity: ghostFade,
    // Convenience: percentage complete (0–1) in case you want to drive other props
    progress: sweepPercent / 100,
  };
}
