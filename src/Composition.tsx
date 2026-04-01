import React, { useEffect } from "react";
import {
  AbsoluteFill,
  interpolate,
  Sequence,
  useCurrentFrame,
  Easing,
  random,
} from "remotion";

import { loadFont } from "@remotion/google-fonts/JetBrainsMono";

const { fontFamily } = loadFont("normal", {
  weights: ["700"],
  subsets: ["latin"],
});

type MyCompProps = {
  fadeDirection: string;
  Text: string;
};

export const FastEaseText: React.FC<MyCompProps> = ({
  fadeDirection,
  Text,
}) => {
  const frame = useCurrentFrame();
  // FAST ease-out curve (snappy start, smooth finish)
  const easeOut = Easing.out(Easing.cubic);
    
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

  const fade = Object.freeze({
    left: interpolate(frame, [0, 20], [-300, 0], {
      easing: easeOut,
      extrapolateRight: "clamp",
    }),
    right: interpolate(frame, [0, 20], [300, 0], {
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
  const opacity = interpolate(frame, [0, 15], [0, 1], {
    easing: easeOut,
    extrapolateRight: "clamp",
  });
  
  const textStyle = {
    color: "#fff",
    fontSize: 100,
    margin: 0,
    fontFamily: fontFamily,
    transform: `${getFadeDirection(fadeDirection)}`,
    opacity,
  };
  return (
    <AbsoluteFill
      style={{
        justifyContent: "center",
        alignItems: "center",
        backgroundColor: "#000",
      }}
    >
      <h1 style={textStyle}>{Text}</h1>
    </AbsoluteFill>
  );
};

function randomFadeDirection(i: number) {
  const mynum = Math.floor(random(i) * 4);
  const directions = ["top", "bottom", "left", "right"];
  return directions[mynum];
}

function myRegex(Text: string): string[] {
  const regex = /\b[\w']+[^\s\w]*/g;
  const splitString = Text?.match(regex) || [];
  return splitString;
}

// --- MAIN COMPOSITION WIRING ---
export type VideoProps = {
  script: string[];
}
export const Video: React.FC<VideoProps> = ({script}) => {
  // const splitString = myRegex("This Video Was Made With Code isn't that cool?");
  const splitString = script || [];
  useEffect(() => {
    console.log(splitString);
  }, []);

  return (
    <AbsoluteFill style={{ backgroundColor: "#0A0A0A" }}>
      {/* Scene 1 runs from frame 0 to 50 */}
      {/*scene 4 my test scene*/}
      {/*<FastEaseText fadeDirection="right" Text="My name is Shammah" />*/}
      {splitString.map((item, i) => (
        <Sequence  key={i} durationInFrames={50} from={15 * i}>
          <FastEaseText
            fadeDirection={randomFadeDirection(i)}
            Text={item}
          />
        </Sequence>
      ))}
    </AbsoluteFill>
  );
};

export default Video;
