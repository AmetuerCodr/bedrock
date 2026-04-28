import { spring, interpolate } from 'remotion';

/**
 * WEIGHT BREATH
 * Animates font-weight from a thin value up to a heavy value and back down,
 * creating a "breathing" pulse. Staggered per word index.
 * Requires a variable font that supports the wght axis (e.g. Inter, Syne).
 *
 * Usage:
 *   words.map((word, i) => {
 *     const { fontWeight, opacity } = weightBreathe(frame, fps, i);
 *     return <span style={{ fontWeight, opacity }}>{word}</span>
 *   })
 *
 * Note: fontWeight must be applied as a number (e.g. style={{ fontWeight: 300 }})
 * not as a font-variation-settings string. React handles the interpolation.
 */
export function weightBreathe(
  frame: number,
  fps: number,
  wordIndex: number,
  options: {
    delayPerWord?: number; // frames between each word's pulse
    fromWeight?: number;   // starting font-weight (thin end)
    toWeight?: number;     // peak font-weight (heavy end)
    holdFrames?: number;   // how long to hold at peak before releasing
    damping?: number;
    stiffness?: number;
  } = {}
) {
  const {
    delayPerWord = 8,
    fromWeight = 100,
    toWeight = 900,
    holdFrames = 12,
    damping = 18,
    stiffness = 80,
  } = options;

  const localFrame = frame - wordIndex * delayPerWord;

  // Spring up to peak weight
  const riseProgress = spring({
    frame: localFrame,
    fps,
    config: { damping, stiffness, mass: 1 },
  });

  // Spring back down after hold
  const fallProgress = spring({
    frame: localFrame - holdFrames,
    fps,
    config: { damping: damping + 4, stiffness: stiffness - 20, mass: 1 },
  });

  // Net weight: rise up, then fall back toward fromWeight
  const weight = interpolate(riseProgress, [0, 1], [fromWeight, toWeight])
    - interpolate(fallProgress, [0, 1], [0, toWeight - fromWeight]);

  const clampedWeight = Math.max(fromWeight, Math.min(toWeight, weight));

  const opacity = interpolate(localFrame, [0, 6], [0, 1], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
  });

  return {
    fontWeight: clampedWeight,
    opacity,
  };
}