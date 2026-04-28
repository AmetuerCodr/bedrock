import { spring, interpolate } from "remotion";

/**
 * SHEAR SNAP
 * Word arrives from an italic skew angle, snaps hard to upright,
 * then micro-overshoots back slightly before settling.
 * Staggered per word index.
 *
 * Usage:
 *   words.map((word, i) => {
 *     const { skewX, translateX, opacity } = shearSnap(frame, fps, i);
 *     return (
 *       <span style={{
 *         transform: `skewX(${skewX}deg) translateX(${translateX}px)`,
 *         opacity,
 *         display: 'inline-block', // required for transform on inline elements
 *       }}>
 *         {word}
 *       </span>
 *     )
 *   })
 */
export function shearSnap(
  frame: number,
  fps: number,
  wordIndex: number,
  options: {
    delayPerWord?: number; // frames between each word
    fromSkew?: number; // starting skew in degrees (negative = lean left)
    overshootSkew?: number; // micro-overshoot skew on settle (opposite direction)
    damping?: number;
    stiffness?: number;
  } = {},
) {
  const {
    delayPerWord = 10,
    fromSkew = -28,
    overshootSkew = 5,
    damping = 16,
    stiffness = 140,
  } = options;

  const localFrame = frame - wordIndex * delayPerWord;

  // Main snap: skew collapses to 0
  const snapProgress = spring({
    frame: localFrame,
    fps,
    config: { damping, stiffness, mass: 0.8 },
  });

  // Micro re-shear: slight overshoot in opposite direction after snap
  const overshootProgress = spring({
    frame: localFrame - 4,
    fps,
    config: { damping: damping + 8, stiffness: stiffness + 30, mass: 0.5 },
  });

  const skewX =
    interpolate(snapProgress, [0, 1], [fromSkew, 0]) +
    interpolate(overshootProgress, [0, 1], [0, overshootSkew]) -
    interpolate(overshootProgress, [0, 1], [0, overshootSkew]); // resolves to 0 at rest

  // Simpler version that actually produces the overshoot feel:
  const baseSkew = interpolate(snapProgress, [0, 1], [fromSkew, 0]);
  const microSkew =
    interpolate(overshootProgress, [0, 1], [0, overshootSkew], {
      extrapolateRight: "clamp",
    }) -
    interpolate(
      spring({
        frame: localFrame - 10,
        fps,
        config: { damping: 20, stiffness: 100 },
      }),
      [0, 1],
      [0, overshootSkew],
      { extrapolateRight: "clamp" },
    );

  // translateX compensates for the visual shift caused by skewing
  const translateX = interpolate(snapProgress, [0, 1], [-12, 0]);

  // Motion blur approximation: slight blur on fast frames, clears on settle
  const blur = interpolate(localFrame, [0, 8], [3, 0], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  const opacity = interpolate(localFrame, [0, 4], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

  return {
    skewX: baseSkew + microSkew,
    translateX,
    blur,
    opacity,
  };
}
