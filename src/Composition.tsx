import React from "react";
import {
  AbsoluteFill,
  interpolate,
  Sequence,
  spring,
  useCurrentFrame,
  useVideoConfig,
  Easing,
} from "remotion";
import { noise3D } from "@remotion/noise";

const OVERSCAN_MARGIN = 100;
const ROWS = 10;
const COLS = 15;

import { loadFont } from "@remotion/google-fonts/Montserrat";

const { fontFamily } = loadFont("normal", {
  weights: ["700"],
  subsets: ["latin"],
});

const NoiseComp: React.FC<{
  speed: number;
  circleRadius: number;
  maxOffset: number;
}> = ({ speed, circleRadius, maxOffset }) => {
  const frame = useCurrentFrame();
  const { height, width } = useVideoConfig();

  return (
    <svg width={width} height={height}>
      {new Array(COLS).fill(0).map((_, i) =>
        new Array(ROWS).fill(0).map((__, j) => {
          const x = i * ((width + OVERSCAN_MARGIN) / COLS);
          const y = j * ((height + OVERSCAN_MARGIN) / ROWS);
          const px = i / COLS;
          const py = j / ROWS;
          const dx = noise3D("x", px, py, frame * speed) * maxOffset;
          const dy = noise3D("y", px, py, frame * speed) * maxOffset;
          const opacity = interpolate(
            noise3D("opacity", i, j, frame * speed),
            [-1, 1],
            [0, 1],
          );

          const key = `${i}-${j}`;

          return (
            <circle
              key={key}
              cx={x + dx}
              cy={y + dy}
              r={circleRadius}
              fill="gray"
              opacity={opacity}
            />
          );
        }),
      )}
    </svg>
  );
};

// --- SCENE 1: Basic Fade In ---
const SceneFade: React.FC = () => {
  const frame = useCurrentFrame();
  // Interpolate maps the frame number (0 to 15) to an opacity value (0 to 1)
  const opacity = interpolate(frame, [0, 15], [0, 1], {
    extrapolateRight: "clamp",
  });

  return (
    <AbsoluteFill
      style={{ justifyContent: "center", alignItems: "center", opacity }}
    >
      <h1
        style={{
          color: "#FFFFFF",
          fontSize: "80px",
          fontFamily: fontFamily,
          margin: 0,
        }}
      >
        Scene 1: Fade
      </h1>
    </AbsoluteFill>
  );
};

// --- SCENE 2: Basic Spring Scale ---
const SceneSpring: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  // Spring creates a bouncy, physics-based value from 0 to 1
  const scale = spring({
    fps,
    frame,
    config: { damping: 100, stiffness: 170 },
  });

  return (
    <AbsoluteFill style={{ justifyContent: "center", alignItems: "center" }}>
      <h1
        style={{
          color: "#D4AF37",
          fontSize: "100px",
          margin: 0,
          transform: `scale(${scale})`,
        }}
      >
        Scene 2: Spring
      </h1>
    </AbsoluteFill>
  );
};

// --- SCENE 3: Basic Mask Slide ---
const SceneSlide: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const progress = spring({
    fps,
    frame,
    config: { damping: 14, stiffness: 150 },
  });
  // Map the spring progress (0 to 1) to a pixel translation (150px down, to 0px)
  const translateY = interpolate(progress, [0, 1], [150, 0]);

  return (
    <AbsoluteFill style={{ justifyContent: "center", alignItems: "center" }}>
      {/* The overflow: 'hidden' box acts as the mask */}
      <div style={{ overflow: "hidden", padding: "10px" }}>
        <h1
          style={{
            color: "#A0A0A0",
            fontSize: "90px",
            margin: 0,
            fontFamily: fontFamily,
            transform: `translateY(${translateY}px)`,
          }}
        >
          Scene 3: Slide
        </h1>
      </div>
    </AbsoluteFill>
  );
};

// SCENE 4

interface MyComp {
  fadeDirection: string;
  Text: string;
}

export const FastEaseText: React.FC<MyComp> = ({ fadeDirection, Text }) => {
  const frame = useCurrentFrame();
  // FAST ease-out curve (snappy start, smooth finish)
  const easeOut = Easing.out(Easing.cubic);

  // slide from left (-300px → 0)
  // const x = interpolate(frame, [0, 20], [-300, 0], {
  //   easing: easeOut,
  //   extrapolateRight: "clamp",
  // });

  // const y = interpolate(frame, [0, 20], [300, 0], {
  //   easing: easeOut,
  //   extrapolateRight: "clamp",
  // });
  
  
  const getFadeDirection = (direction: string): string => {
    switch (direction) {
      case "left":
        return `translateX(${fade.left}px)`;
      case "right":
        return `translateX(${fade.right}px)`;
      case "top":
        return `translateY(${fade.top}px)`;
      case "bottom":
        return `translateY(${fade.bottom}px)`;
      default:
        return `translateX(${fade.left}px)`;
    }
  };

  // const handleFadeDirection = (direction: string) => {
  //   getFadeDirection(direction)
  // }
  const fade = Object.freeze({
    left: interpolate(frame, [0, 20], [ 300, 0], {
      easing: easeOut,
      extrapolateRight: "clamp",
    }),
    right: interpolate(frame, [0, 20], [-300, 0], {
      easing: easeOut,
      extrapolateRight: "clamp",
    }),
    top: interpolate(frame, [0, 20], [-300, 0], {
      easing: easeOut,
      extrapolateRight: "clamp",
    }),
    bottom: interpolate(frame, [0, 20], [300, 0], {
      easing: easeOut,
      extrapolateRight: "clamp",
    }),
  });
  
  


  // fade in (slightly faster than movement)
  const opacity = interpolate(frame, [0, 15], [0, 1], {
    easing: easeOut,
    extrapolateRight: "clamp",
  });

  return (
    <AbsoluteFill
      style={{
        justifyContent: "center",
        alignItems: "center",

        backgroundColor: "#000",
      }}
    >
      <h1
        style={{
          color: "#fff",
          fontSize: 100,
          margin: 0,
          fontFamily: fontFamily,
          transform: `${getFadeDirection(fadeDirection)}`,
          opacity,
        }}
      >
        {Text}
      </h1>
    </AbsoluteFill>
  );
};

// --- MAIN COMPOSITION WIRING ---
export const Video: React.FC = () => {
  return (
    <AbsoluteFill style={{ backgroundColor: "#0A0A0A" }}>
      {/* Scene 1 runs from frame 0 to 50 */}
      {/*scene 4 my test scene*/}
      <Sequence from={150} durationInFrames={50}>
        <FastEaseText fadeDirection="left" Text="hello, world!" />
      </Sequence>
    </AbsoluteFill>
  );
};

export default Video;

// <Sequence durationInFrames={50}>
//   <SceneFade />
// </Sequence>

// {/* Scene 2 runs from frame 50 to 100 */}
// <Sequence from={50} durationInFrames={50}>
//   <SceneSpring />
// </Sequence>

// {/* Scene 3 runs from frame 100 to 150 */}
// <Sequence from={100} durationInFrames={50}>
//   <SceneSlide />
// </Sequence>
